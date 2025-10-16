#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::{DateTime, Local};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Stdio;
use tauri::{Emitter, Manager};
use tauri_plugin_shell::ShellExt;

mod auth;
mod constants;
mod db;
mod downloads;

use auth::*;
use db::*;
use downloads::*;

// Additional imports for model management
use db::{
    get_default_model_preferences, load_model_preferences, model_preferences_exist,
    save_model_preferences, ModelPreferences,
};

/// Helper to get database connection
fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join("medical_notes.db");

    initialize_database(&db_path).map_err(|e| format!("Failed to initialize database: {}", e))
}

/// Helper function to get the DEK from the database with password
async fn get_dek_from_auth_with_password(
    app: &tauri::AppHandle,
    password: &str,
) -> Result<Vec<u8>, String> {
    let conn = get_db_connection(app)?;

    // Check if auth exists in database
    if !check_auth_exists_in_db(&conn) {
        return Err("No authentication data found".to_string());
    }

    let auth_file = load_auth_from_db(&conn)
        .map_err(|e| format!("Failed to load auth from database: {}", e))?;

    get_dek(&auth_file, password).map_err(|e| format!("Failed to decrypt DEK: {}", e))
}

/// Convert PatientNote to EncryptedNote
fn encrypt_note(note: &PatientNote, dek: &[u8]) -> Result<EncryptedNote, String> {
    // Serialize the note to JSON
    let json_data =
        serde_json::to_string(note).map_err(|e| format!("Failed to serialize note: {}", e))?;

    // Encrypt the entire JSON blob
    let (encrypted_data, nonce) =
        encrypt_data(&json_data, dek).map_err(|e| format!("Failed to encrypt note data: {}", e))?;

    Ok(EncryptedNote {
        id: note.id.clone(),
        encrypted_data,
        nonce,
        created_at: note.created_at,
    })
}

/// Convert EncryptedNote to PatientNote
fn decrypt_note(encrypted_note: &EncryptedNote, dek: &[u8]) -> Result<PatientNote, String> {
    // Decrypt the entire JSON blob
    let json_data = decrypt_data(&encrypted_note.encrypted_data, dek, &encrypted_note.nonce)
        .map_err(|e| format!("Failed to decrypt note data: {}", e))?;

    // Deserialize the JSON back to PatientNote
    let note: PatientNote = serde_json::from_str(&json_data)
        .map_err(|e| format!("Failed to deserialize note: {}", e))?;

    Ok(note)
}

#[derive(Serialize)]
struct TranscriptionResult {
    success: bool,
    transcript: String,
    error: Option<String>,
}

#[derive(Serialize)]
struct MedicalNoteResult {
    success: bool,
    note: String,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct PatientNote {
    id: String,
    first_name: String,
    last_name: String,
    date_of_birth: String,
    note_type: String,
    transcript: String,
    medical_note: String,
    created_at: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
struct EncryptedNote {
    id: String,
    encrypted_data: String,
    nonce: String,
    created_at: DateTime<Local>,
}

#[derive(Serialize)]
struct NoteResult {
    success: bool,
    note_id: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct LoadNotesResult {
    success: bool,
    notes: Vec<PatientNote>,
    error: Option<String>,
}

#[tauri::command]
async fn ensure_app_directory(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;

    // Create the directory if it doesn't exist
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app directory: {}", e))?;
    }

    Ok(app_data_dir.to_string_lossy().to_string())
}

#[tauri::command]
async fn validate_audio_file(audio_path: String) -> Result<String, String> {
    println!("Validating audio file: {}", audio_path);

    // Check if file exists
    let path = std::path::Path::new(&audio_path);
    if !path.exists() {
        return Err(format!("Audio file does not exist: {}", audio_path));
    }

    // Check file size
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();
    println!("Audio file size: {} bytes", file_size);

    if file_size < 44 {
        return Err("Audio file is too small (less than WAV header size)".to_string());
    }

    if file_size < 1000 {
        return Err("Audio file is suspiciously small - may contain no audio data".to_string());
    }

    // Try to read WAV header if it's a WAV file
    if audio_path.ends_with(".wav") {
        if let Ok(mut file) = std::fs::File::open(path) {
            use std::io::Read;
            let mut header = [0u8; 12];
            if file.read_exact(&mut header).is_ok() {
                let riff = String::from_utf8_lossy(&header[0..4]);
                let wave = String::from_utf8_lossy(&header[8..12]);

                if riff != "RIFF" || wave != "WAVE" {
                    return Err("Invalid WAV file format".to_string());
                }

                println!("Valid WAV header detected");
            }
        }
    }

    Ok(format!("Audio file validated: {} bytes", file_size))
}

#[tauri::command]
async fn transcribe_audio(
    app: tauri::AppHandle,
    audio_path: String,
) -> Result<TranscriptionResult, String> {
    println!("Starting transcription for: {}", audio_path);

    // Emit transcription started event
    app.emit("transcription-progress", "Starting transcription...")
        .ok();

    // Validate audio file first
    if let Err(validation_error) = validate_audio_file(audio_path.clone()).await {
        return Ok(TranscriptionResult {
            success: false,
            transcript: String::new(),
            error: Some(format!("Audio validation failed: {}", validation_error)),
        });
    }

    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    println!("App data directory: {:?}", app_data_dir);

    // Determine the correct whisperfile executable
    let whisperfile_name = if cfg!(target_os = "windows") {
        "whisperfile.exe"
    } else {
        "whisperfile"
    };

    // Try different possible locations for the whisperfile
    let whisperfile_paths = [
        // Production: app data directory (where setup wizard downloads them)
        app_data_dir.join("binaries").join(whisperfile_name),
        // Development: relative to project root
        PathBuf::from("binaries").join(whisperfile_name),
    ];

    let mut whisperfile_path = None;
    for path in &whisperfile_paths {
        println!("Checking whisperfile path: {:?}", path);
        if path.exists() {
            whisperfile_path = Some(path.clone());
            println!("Found whisperfile at: {:?}", path);
            break;
        }
    }

    let whisperfile_path = match whisperfile_path {
        Some(path) => path,
        None => {
            println!("Whisperfile not found in any of these locations:");
            for path in &whisperfile_paths {
                println!("  {:?}", path);
            }
            return Ok(TranscriptionResult {
                success: false,
                transcript: String::new(),
                error: Some(format!(
                    "Whisperfile not found. Tried: {:?}",
                    whisperfile_paths
                )),
            });
        }
    };

    // Load model preferences from database
    let conn = get_db_connection(&app)?;
    let preferred_model = match load_model_preferences(&conn) {
        Ok(prefs) => {
            println!(
                "Using preferred whisper model: {}",
                prefs.whisper_model_filename
            );
            Some(prefs.whisper_model_filename)
        }
        Err(_) => {
            println!("No model preferences found, using default model search");
            None
        }
    };

    // Build list of model names to try, prioritizing the preferred model
    let default_model_names = [
        "whisper-tiny.en.gguf",
        "ggml-tiny.en.bin",
        "whisper-tiny.en.bin",
        "whisper-small.en.gguf",
        "ggml-small.en.bin",
    ];

    let mut model_names_to_try = Vec::new();

    // Add preferred model first if it exists and is not already in the default list
    if let Some(ref preferred) = preferred_model {
        model_names_to_try.push(preferred.as_str());
    }

    // Add default models that aren't the preferred model
    for model_name in &default_model_names {
        if Some(model_name.to_string()) != preferred_model {
            model_names_to_try.push(model_name);
        }
    }

    let model_paths = [
        // Production: app data directory (where setup wizard downloads them)
        app_data_dir.join("binaries").join("models"),
        // Development: relative to project root
        PathBuf::from("binaries").join("models"),
    ];

    let mut model_path = None;
    'outer: for base_path in &model_paths {
        for model_name in &model_names_to_try {
            let test_path = base_path.join(model_name);
            println!("Checking model path: {:?}", test_path);
            if test_path.exists() {
                model_path = Some(test_path);
                println!("Found model at: {:?}", model_path.as_ref().unwrap());
                break 'outer;
            }
        }
    }

    let model_path = match model_path {
        Some(path) => path,
        None => {
            println!("Model not found in any location with any name");
            return Ok(TranscriptionResult {
                success: false,
                transcript: String::new(),
                error: Some("Whisper model not found. Check that model files exist in binaries/models/ directory".to_string()),
            });
        }
    };

    // Check if the audio format is supported by whisperfile
    let is_supported_format = audio_path.ends_with(".wav")
        || audio_path.ends_with(".mp3")
        || audio_path.ends_with(".flac")
        || audio_path.ends_with(".ogg");

    if !is_supported_format {
        let file_extension = audio_path.split('.').last().unwrap_or("unknown");
        return Ok(TranscriptionResult {
            success: false,
            transcript: String::new(),
            error: Some(format!(
                "Audio format '.{}' is not supported by whisperfile. Supported formats: .wav, .mp3, .flac, .ogg",
                file_extension
            )),
        });
    }

    println!("Audio file for transcription: {}", audio_path);

    // Execute whisperfile with correct arguments based on the documentation
    println!(
        "Executing whisperfile with args: -m {:?} -f {} --no-prints",
        model_path, audio_path
    );

    // Emit progress update
    app.emit(
        "transcription-progress",
        "Processing audio with Whisper model...",
    )
    .ok();

    let output = app
        .shell()
        .command(&whisperfile_path)
        .args([
            "-m",
            &model_path.to_string_lossy(),
            "-f",
            &audio_path,
            "--no-prints", // Suppress debug output - this is the key flag for whisperfile
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute whisperfile: {}", e))?;

    println!("Whisperfile exit status: {:?}", output.status);
    println!("Whisperfile stdout length: {}", output.stdout.len());
    println!("Whisperfile stderr length: {}", output.stderr.len());

    // Print stderr to see what whisperfile is saying
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        println!("Whisperfile stderr: {}", stderr_str);

        // Check for specific error patterns
        if stderr_str.contains("failed to read pcm frames")
            || stderr_str.contains("At end otalerror")
        {
            return Ok(TranscriptionResult {
                success: false,
                transcript: String::new(),
                error: Some("Audio file appears to be corrupted or empty. Try recording again with a longer duration and ensure your microphone is working.".to_string()),
            });
        }
    }

    if output.status.success() {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        println!("Raw whisperfile output: {}", stdout_str);

        let transcript = parse_whisper_output(&stdout_str);
        println!("Parsed transcript: {}", transcript);

        // Emit the transcript as it's being processed
        app.emit("transcription-text", &transcript).ok();

        // Check for blank audio detection
        if transcript.contains("[BLANK_AUDIO]") || transcript.trim().is_empty() {
            return Ok(TranscriptionResult {
                success: false,
                transcript: String::new(),
                error: Some("No speech detected in audio. Please ensure you speak clearly into the microphone and try recording again.".to_string()),
            });
        }

        Ok(TranscriptionResult {
            success: true,
            transcript,
            error: None,
        })
    } else {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        println!("Whisperfile error: {}", stderr_str);

        Ok(TranscriptionResult {
            success: false,
            transcript: String::new(),
            error: Some(format!("Transcription failed: {}", stderr_str)),
        })
    }
}

#[tauri::command]
async fn generate_medical_note(
    app: tauri::AppHandle,
    transcript: String,
    note_type: String,
) -> Result<MedicalNoteResult, String> {
    println!(
        "Starting medical note generation for transcript: {}",
        transcript
    );

    // Emit note generation started event
    let note_type_display = if note_type == "soap" { "SOAP" } else { "Full" };
    app.emit(
        "note-generation-progress",
        format!("Generating {} medical note...", note_type_display),
    )
    .ok();

    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;

    // Determine the correct llamafile executable
    let llamafile_name = if cfg!(target_os = "windows") {
        "llamafile.exe"
    } else {
        "llamafile"
    };

    // Get the current working directory to build absolute paths
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let project_root = if current_dir.ends_with("src-tauri") {
        current_dir.parent().unwrap_or(&current_dir).to_path_buf()
    } else {
        current_dir
    };

    // Try different possible locations for the llamafile with absolute paths
    let llamafile_paths = [
        // Production: app data directory (where setup wizard downloads them)
        app_data_dir.join("binaries").join(llamafile_name),
        // Development: absolute from project root
        project_root.join("binaries").join(llamafile_name),
    ];

    let mut llamafile_path = None;
    for path in &llamafile_paths {
        println!("Checking llamafile path: {:?}", path);
        if path.exists() {
            llamafile_path = Some(path.clone());
            println!("Found llamafile at: {:?}", path);
            break;
        }
    }

    let llamafile_path = match llamafile_path {
        Some(path) => path,
        None => {
            return Ok(MedicalNoteResult {
                success: false,
                note: String::new(),
                error: Some(format!("Llamafile not found. Tried: {:?}", llamafile_paths)),
            });
        }
    };

    // Load model preferences from database
    let conn = get_db_connection(&app)?;
    let preferred_model = match load_model_preferences(&conn) {
        Ok(prefs) => {
            println!(
                "Using preferred MedLlama model: {}",
                prefs.med_llama_filename
            );
            Some(prefs.med_llama_filename)
        }
        Err(_) => {
            println!("No model preferences found, using default model search");
            None
        }
    };

    // Build list of model names to try, prioritizing the preferred model
    let default_model_names = [
        "med_llama.gguf",
        "llama-2-7b-chat.gguf",
        "llama-2-13b-chat.gguf",
        "mistral-7b-instruct.gguf",
        "openchat-3.5.gguf",
    ];

    let mut model_names_to_try = Vec::new();

    // Add preferred model first if it exists and is not already in the default list
    if let Some(ref preferred) = preferred_model {
        model_names_to_try.push(preferred.as_str());
    }

    // Add default models that aren't the preferred model
    for model_name in &default_model_names {
        if Some(model_name.to_string()) != preferred_model {
            model_names_to_try.push(model_name);
        }
    }

    let model_paths = [
        // Production: app data directory (where setup wizard downloads them)
        app_data_dir.join("binaries").join("models"),
        // Development: absolute paths from project root
        project_root.join("binaries").join("models"),
    ];

    let mut model_path = None;
    'outer_llm: for base_path in &model_paths {
        for model_name in &model_names_to_try {
            let test_path = base_path.join(model_name);
            println!("Checking LLM model path: {:?}", test_path);
            if test_path.exists() {
                // Convert to absolute path
                let absolute_path = test_path
                    .canonicalize()
                    .unwrap_or_else(|_| test_path.clone());
                model_path = Some(absolute_path);
                println!("Found LLM model at: {:?}", model_path.as_ref().unwrap());
                break 'outer_llm;
            }
        }
    }

    let model_path = match model_path {
        Some(path) => path,
        None => {
            println!("Project root: {:?}", project_root);
            println!("Available paths checked:");
            for path in &model_paths {
                println!("  - {:?} (exists: {})", path, path.exists());
            }
            return Ok(MedicalNoteResult {
                success: false,
                note: String::new(),
                error: Some(format!("LLM model not found. Project root: {:?}. Check that model files exist in binaries/models/ directory", project_root)),
            });
        }
    };

    // Use the correct chat template for your model with separated system and user prompts
    let (system_prompt, user_prompt_template, assistant_start) = if note_type == "soap" {
        (
            constants::SOAP_SYSTEM_PROMPT,
            constants::SOAP_USER_PROMPT_TEMPLATE,
            "<soap_note>",
        )
    } else {
        (
            constants::FULL_MEDICAL_SYSTEM_PROMPT,
            constants::FULL_MEDICAL_USER_PROMPT_TEMPLATE,
            "",
        )
    };

    // Format the user prompt with the transcript
    let user_prompt = user_prompt_template.replace("{transcript}", &transcript);

    // Combine system and user prompts with proper chat template formatting
    let prompt = format!(
        "<|begin_of_text|><|start_header_id|>system<|end_header_id|>{system_prompt}<|eot_id|><|start_header_id|>user<|end_header_id|>{user_prompt}<|eot_id|><|start_header_id|>assistant<|end_header_id|>{assistant_start}",
        system_prompt = system_prompt,
        user_prompt = user_prompt,
        assistant_start = assistant_start
    );

    println!("=== PROMPT BEING SENT ===");
    println!("{}", prompt);
    println!("=== END PROMPT ===");

    // Execute llamafile with supported parameters only
    println!(
        "Executing llamafile with absolute model path: {:?}",
        model_path
    );

    let mut cmd = std::process::Command::new(&llamafile_path);
    cmd.current_dir(&project_root)
        .args([
            "-m",
            &model_path.to_string_lossy(),
            "--temp",
            constants::TEMPERATURE, // Low temp for consistent output
            "--top-p",
            "0.95",
            // "--top-k", "30",
            // "--repeat-penalty", "1.05", // Prevent repetition
            "-n",
            "4096", // Limit output length
            // "--threads", "4",
            // "--ctx-size", "4096",
            "--no-display-prompt", // Don't echo prompt
            // "--batch-size", "512",
            "--log-disable", // Disable logging
            "-p",
            &prompt,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    println!("Starting llamafile process...");
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute llamafile: {}", e))?;

    // Stream the output
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let reader = BufReader::new(stdout);
    let mut accumulated_output = String::new();
    let mut is_generating = false;

    for line in reader.lines() {
        if let Ok(line) = line {
            accumulated_output.push_str(&line);
            accumulated_output.push('\n');

            // Start streaming after we see the initial pattern
            if !is_generating && (line.contains("S:") || line.contains("1. Presenting Illness")) {
                is_generating = true;
            }

            if is_generating {
                // Emit the raw output directly without cleaning for real-time display
                app.emit("note-generation-stream", &line).ok();
            }
        }
    }

    // Wait for the process to complete
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for llamafile: {}", e))?;

    println!("Llamafile process completed");

    if status.success() {
        // Clean the final output
        let note = clean_llm_output(&accumulated_output);
        println!("Generated note length: {}", note.len());

        if note.trim().is_empty() {
            return Ok(MedicalNoteResult {
                success: false,
                note: String::new(),
                error: Some(
                    "LLM produced empty output. Model may have failed to generate response."
                        .to_string(),
                ),
            });
        }

        // Send the final cleaned note
        app.emit("note-generation-complete", &note).ok();

        Ok(MedicalNoteResult {
            success: true,
            note,
            error: None,
        })
    } else {
        Ok(MedicalNoteResult {
            success: false,
            note: String::new(),
            error: Some("Note generation failed".to_string()),
        })
    }
}

fn parse_whisper_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut transcript_parts = Vec::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Whisperfile output format: [00:00:00.000 --> 00:00:05.000] Transcript text
        if let Some(bracket_end) = line.find("] ") {
            if line.starts_with('[') {
                let text_part = &line[bracket_end + 2..];
                if !text_part.trim().is_empty() && !text_part.contains("[BLANK_AUDIO]") {
                    transcript_parts.push(text_part.trim());
                }
            }
        } else if !line.starts_with('[') && !line.contains("->") && !line.contains("[BLANK_AUDIO]")
        {
            // Handle lines without timestamps
            transcript_parts.push(line);
        }
    }

    transcript_parts.join(" ")
}

fn clean_llm_output(output: &str) -> String {
    let mut result = output.to_string();

    // Remove common llamafile artifacts and stop tokens
    let artifacts_to_remove = [
        "<|begin_of_text|>",
        "<|start_header_id|>",
        "<|end_header_id|>",
        "<|eot_id|>",
        "<|end_of_text|>",
        "user",
        "assistant",
    ];

    for artifact in &artifacts_to_remove {
        result = result.replace(artifact, "");
    }

    let mut cleaned = result.trim().to_string();

    // If the output doesn't start with "S:", add it back (only for SOAP notes)
    if !cleaned.starts_with("S:")
        && !cleaned.is_empty()
        && !cleaned.contains("Presenting Illness")
        && !cleaned.contains("History of Presenting Illness")
    {
        cleaned = format!("S: {}", cleaned);
    }

    // Handle case where model might continue generating beyond SOAP note
    // Look for natural stopping points or repetitive content
    let lines: Vec<&str> = cleaned.lines().collect();
    let mut final_lines = Vec::new();
    let mut last_section = "";

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines at the start
        if trimmed.is_empty() && final_lines.is_empty() {
            continue;
        }

        // Check for SOAP section headers
        if trimmed.starts_with("S:")
            || trimmed.starts_with("O:")
            || trimmed.starts_with("A:")
            || trimmed.starts_with("P:")
        {
            last_section = &trimmed[0..2];
            final_lines.push(trimmed);
            continue;
        }

        // Skip obvious artifacts and repetitive content
        if trimmed.contains("Create a SOAP")
            || trimmed.contains("medical conversation")
            || trimmed.contains("Provide only")
            || trimmed.len() < 3
        {
            continue;
        }

        // Stop if we see the model continuing the conversation
        if trimmed.contains("Dr. Thomas") && trimmed.contains(":")
            || trimmed.contains("Susan") && trimmed.contains(":")
            || trimmed.contains("Patient") && trimmed.contains(":")
        {
            break;
        }

        // Stop if we see obvious model artifacts
        if trimmed.contains("**")
            || trimmed.contains("###")
            || trimmed.starts_with("---")
            || trimmed.contains("```")
        {
            break;
        }

        // Add content lines
        if !trimmed.is_empty() {
            final_lines.push(trimmed);
        } else if !final_lines.is_empty() {
            // Preserve spacing within SOAP note
            final_lines.push("");
        }

        // Stop if we've completed all SOAP sections and see repetitive content
        if final_lines.len() > 10 && last_section == "P:" {
            // Check if this line repeats previous content
            let line_words: Vec<&str> = trimmed.split_whitespace().collect();
            if line_words.len() > 3 {
                let joined_prev = final_lines.join(" ").to_lowercase();
                let current_line = trimmed.to_lowercase();
                if joined_prev.contains(&current_line) {
                    break; // Stop on repetitive content
                }
            }
        }
    }

    // Join and final cleanup
    let result = final_lines.join("\n").trim().to_string();

    // Ensure we have reasonable SOAP content
    if result.len() < 50 || !result.contains("S:") {
        println!("Warning: Generated SOAP note seems incomplete or malformed");
    }

    result
}

#[tauri::command]
#[allow(non_snake_case)]
async fn create_patient_note(
    app: tauri::AppHandle,
    password: String,
    firstName: String,
    lastName: String,
    dateOfBirth: String,
    noteType: String,
    transcript: String,
    medicalNote: String,
) -> Result<NoteResult, String> {
    println!("Creating patient note for {} {}", firstName, lastName);

    // Get the DEK using the password
    let dek = get_dek_from_auth_with_password(&app, &password).await?;
    let conn = get_db_connection(&app)?;

    // Generate unique note ID
    let note_id = format!("{}", chrono::Local::now().timestamp_millis());
    let created_at = chrono::Local::now();

    let patient_note = PatientNote {
        id: note_id.clone(),
        first_name: firstName,
        last_name: lastName,
        date_of_birth: dateOfBirth,
        note_type: noteType,
        transcript,
        medical_note: medicalNote,
        created_at,
    };

    // Encrypt the note
    let encrypted_note = encrypt_note(&patient_note, &dek)?;

    // Convert to database format and save
    let encrypted_note_data = EncryptedNoteData {
        id: encrypted_note.id.clone(),
        encrypted_data: encrypted_note.encrypted_data,
        nonce: encrypted_note.nonce,
        created_at: encrypted_note.created_at,
    };

    save_encrypted_note(&conn, &encrypted_note_data)
        .map_err(|e| format!("Failed to save note to database: {}", e))?;

    println!(
        "Encrypted note created successfully in database: {}",
        note_id
    );

    Ok(NoteResult {
        success: true,
        note_id: Some(note_id),
        error: None,
    })
}

#[tauri::command]
#[allow(non_snake_case)]
async fn update_patient_note(
    app: tauri::AppHandle,
    password: String,
    noteId: String,
    firstName: String,
    lastName: String,
    dateOfBirth: String,
    noteType: String,
    transcript: String,
    medicalNote: String,
) -> Result<NoteResult, String> {
    println!(
        "Updating patient note {} for {} {}",
        noteId, firstName, lastName
    );

    // Get the DEK using the password
    let dek = get_dek_from_auth_with_password(&app, &password).await?;
    let conn = get_db_connection(&app)?;

    // Check if the note exists in database
    if !note_exists(&conn, &noteId).map_err(|e| format!("Failed to check note existence: {}", e))? {
        return Err(format!("Note not found: {}", noteId));
    }

    // Load existing encrypted note to preserve creation date
    let existing_encrypted_note = load_encrypted_note_by_id(&conn, &noteId)
        .map_err(|e| format!("Failed to load existing note: {}", e))?;

    // Create updated note with existing creation date
    let updated_note = PatientNote {
        id: noteId.clone(),
        first_name: firstName,
        last_name: lastName,
        date_of_birth: dateOfBirth,
        note_type: noteType,
        transcript,
        medical_note: medicalNote,
        created_at: existing_encrypted_note.created_at, // Preserve original creation date
    };

    // Encrypt the updated note
    let encrypted_updated_note = encrypt_note(&updated_note, &dek)?;

    // Convert to database format and save
    let encrypted_note_data = EncryptedNoteData {
        id: encrypted_updated_note.id.clone(),
        encrypted_data: encrypted_updated_note.encrypted_data,
        nonce: encrypted_updated_note.nonce,
        created_at: encrypted_updated_note.created_at,
    };

    save_encrypted_note(&conn, &encrypted_note_data)
        .map_err(|e| format!("Failed to save updated note to database: {}", e))?;

    println!(
        "Encrypted note updated successfully in database: {}",
        noteId
    );

    Ok(NoteResult {
        success: true,
        note_id: Some(noteId),
        error: None,
    })
}

#[tauri::command]
async fn load_patient_notes(
    app: tauri::AppHandle,
    password: String,
) -> Result<LoadNotesResult, String> {
    println!("Loading patient notes...");

    // Get the DEK using the password
    let dek = get_dek_from_auth_with_password(&app, &password).await?;
    let conn = get_db_connection(&app)?;

    // Load all encrypted notes from database
    let encrypted_notes = load_all_encrypted_notes(&conn)
        .map_err(|e| format!("Failed to load notes from database: {}", e))?;

    let mut notes = Vec::new();

    // Decrypt all notes
    for encrypted_note in encrypted_notes {
        let encrypted_note_for_decrypt = EncryptedNote {
            id: encrypted_note.id,
            encrypted_data: encrypted_note.encrypted_data,
            nonce: encrypted_note.nonce,
            created_at: encrypted_note.created_at,
        };

        match decrypt_note(&encrypted_note_for_decrypt, &dek) {
            Ok(note) => notes.push(note),
            Err(e) => println!("Failed to decrypt note: {}", e),
        }
    }

    println!("Loaded {} notes from database", notes.len());

    Ok(LoadNotesResult {
        success: true,
        notes,
        error: None,
    })
}

#[tauri::command]
async fn delete_patient_note(app: tauri::AppHandle, note_id: String) -> Result<bool, String> {
    println!("Deleting patient note: {}", note_id);

    let conn = get_db_connection(&app)?;

    let deleted = delete_note_by_id(&conn, &note_id)
        .map_err(|e| format!("Failed to delete note from database: {}", e))?;

    if deleted {
        println!("Note deleted successfully from database");
        Ok(true)
    } else {
        Err(format!("Note not found in database: {}", note_id))
    }
}

// Authentication Tauri Commands

#[tauri::command]
async fn check_auth_status(app: tauri::AppHandle) -> Result<AuthResponse, String> {
    let conn = match get_db_connection(&app) {
        Ok(conn) => conn,
        Err(e) => {
            return Ok(AuthResponse {
                success: false,
                message: format!("Failed to connect to database: {}", e),
                user: None,
            });
        }
    };

    // Check if auth exists in database
    if !check_auth_exists_in_db(&conn) {
        return Ok(AuthResponse {
            success: false,
            message: "No authentication data found".to_string(),
            user: None,
        });
    }

    match load_auth_from_db(&conn) {
        Ok(auth_file) => Ok(AuthResponse {
            success: true,
            message: "Authentication data exists".to_string(),
            user: Some(UserInfo {
                user_id: auth_file.user_id,
                username: auth_file.user.username,
            }),
        }),
        Err(e) => Ok(AuthResponse {
            success: false,
            message: format!("Failed to load auth from database: {}", e),
            user: None,
        }),
    }
}

#[tauri::command]
async fn create_user_account_command(
    app: tauri::AppHandle,
    request: CreateUserRequest,
) -> Result<AuthResponse, String> {
    let conn = get_db_connection(&app)?;

    // Check if auth already exists in database
    if check_auth_exists_in_db(&conn) {
        return Ok(AuthResponse {
            success: false,
            message: "User account already exists".to_string(),
            user: None,
        });
    }

    match create_user_account(request.username.clone(), request.password) {
        Ok(auth_file) => match save_auth_to_db(&conn, &auth_file) {
            Ok(_) => Ok(AuthResponse {
                success: true,
                message: "User account created successfully".to_string(),
                user: Some(UserInfo {
                    user_id: auth_file.user_id,
                    username: auth_file.user.username,
                }),
            }),
            Err(e) => Ok(AuthResponse {
                success: false,
                message: format!("Failed to save auth to database: {}", e),
                user: None,
            }),
        },
        Err(e) => Ok(AuthResponse {
            success: false,
            message: format!("Failed to create user account: {}", e),
            user: None,
        }),
    }
}

#[tauri::command]
async fn authenticate_user_command(
    app: tauri::AppHandle,
    request: AuthenticateRequest,
) -> Result<AuthResponse, String> {
    let conn = get_db_connection(&app)?;

    if !check_auth_exists_in_db(&conn) {
        return Ok(AuthResponse {
            success: false,
            message: "No authentication data found".to_string(),
            user: None,
        });
    }

    match load_auth_from_db(&conn) {
        Ok(auth_file) => match authenticate_user(&auth_file, &request.password) {
            Ok(true) => Ok(AuthResponse {
                success: true,
                message: "Authentication successful".to_string(),
                user: Some(UserInfo {
                    user_id: auth_file.user_id,
                    username: auth_file.user.username,
                }),
            }),
            Ok(false) => Ok(AuthResponse {
                success: false,
                message: "Invalid password".to_string(),
                user: None,
            }),
            Err(e) => Ok(AuthResponse {
                success: false,
                message: format!("Authentication error: {}", e),
                user: None,
            }),
        },
        Err(e) => Ok(AuthResponse {
            success: false,
            message: format!("Failed to load auth from database: {}", e),
            user: None,
        }),
    }
}

#[tauri::command]
async fn get_user_info_command(app: tauri::AppHandle) -> Result<AuthResponse, String> {
    let conn = get_db_connection(&app)?;

    if !check_auth_exists_in_db(&conn) {
        return Ok(AuthResponse {
            success: false,
            message: "No authentication data found".to_string(),
            user: None,
        });
    }

    match load_auth_from_db(&conn) {
        Ok(auth_file) => Ok(AuthResponse {
            success: true,
            message: "User info retrieved".to_string(),
            user: Some(UserInfo {
                user_id: auth_file.user_id,
                username: auth_file.user.username,
            }),
        }),
        Err(e) => Ok(AuthResponse {
            success: false,
            message: format!("Failed to load auth from database: {}", e),
            user: None,
        }),
    }
}

#[tauri::command]
async fn delete_audio_file(audio_path: String) -> Result<bool, String> {
    println!("Deleting audio file: {}", audio_path);

    let path = std::path::Path::new(&audio_path);

    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete audio file: {}", e))?;
        println!("Audio file deleted successfully");
        Ok(true)
    } else {
        println!(
            "Audio file not found (may have already been deleted): {}",
            audio_path
        );
        Ok(true) // Return true anyway since the file is gone
    }
}

// Setup and Download Commands

#[tauri::command]
async fn check_setup_status(app: tauri::AppHandle) -> Result<bool, String> {
    let conn = get_db_connection(&app)?;
    is_setup_completed(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_required_models_list() -> Result<Vec<ModelDownloadInfo>, String> {
    Ok(get_required_models())
}

#[tauri::command]
async fn check_models_downloaded(
    app: tauri::AppHandle,
) -> Result<Vec<(ModelDownloadInfo, bool)>, String> {
    // Get user preferences to determine which models to check
    let conn = get_db_connection(&app)?;
    let preferences = if model_preferences_exist(&conn).map_err(|e| e.to_string())? {
        load_model_preferences(&conn).map_err(|e| e.to_string())?
    } else {
        get_default_model_preferences()
    };

    check_models_exist_with_preferences(&app, &preferences).await
}

#[tauri::command]
async fn check_all_models_installed(app: tauri::AppHandle) -> Result<bool, String> {
    check_all_models_present(&app).await
}

#[tauri::command]
async fn get_models_info_command(app: tauri::AppHandle) -> Result<Vec<ModelInfo>, String> {
    // Get user preferences to determine which models to check
    let conn = get_db_connection(&app)?;
    let preferences = if model_preferences_exist(&conn).map_err(|e| e.to_string())? {
        load_model_preferences(&conn).map_err(|e| e.to_string())?
    } else {
        get_default_model_preferences()
    };

    get_models_info_with_preferences(&app, &preferences).await
}

#[tauri::command]
async fn download_model_file(
    app: tauri::AppHandle,
    model: ModelDownloadInfo,
) -> Result<String, String> {
    match download_model(&app, model).await {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Download failed: {}", e)),
    }
}

#[tauri::command]
async fn complete_setup(app: tauri::AppHandle) -> Result<bool, String> {
    let conn = get_db_connection(&app)?;
    mark_setup_completed(&conn).map_err(|e| e.to_string())?;
    Ok(true)
}

// Model Management Commands

#[tauri::command]
async fn get_model_preferences_command(app: tauri::AppHandle) -> Result<ModelPreferences, String> {
    let conn = get_db_connection(&app)?;

    // Check if preferences exist in database
    if !model_preferences_exist(&conn).map_err(|e| e.to_string())? {
        // Return default preferences if none exist
        Ok(get_default_model_preferences())
    } else {
        // Load from database
        load_model_preferences(&conn).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn save_model_preferences_command(
    app: tauri::AppHandle,
    preferences: ModelPreferences,
) -> Result<bool, String> {
    let conn = get_db_connection(&app)?;

    // Update timestamp
    let mut prefs = preferences.clone();
    prefs.updated_at = chrono::Local::now().to_rfc3339();

    save_model_preferences(&conn, &prefs).map_err(|e| e.to_string())?;
    Ok(true)
}

#[derive(Serialize)]
struct DownloadedModel {
    filename: String,
    size_bytes: u64,
    path: String,
}

#[tauri::command]
async fn list_downloaded_models(app: tauri::AppHandle) -> Result<Vec<DownloadedModel>, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let models_dir = app_data_dir.join("binaries").join("models");

    // Check if directory exists
    if !models_dir.exists() {
        return Ok(Vec::new());
    }

    let mut models = Vec::new();

    // Read directory entries
    let entries =
        fs::read_dir(&models_dir).map_err(|e| format!("Failed to read models directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Only include files (not directories)
        if path.is_file() {
            let metadata =
                fs::metadata(&path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            models.push(DownloadedModel {
                filename: filename.clone(),
                size_bytes: metadata.len(),
                path: path.to_string_lossy().to_string(),
            });
        }
    }

    // Sort by filename for consistent ordering
    models.sort_by(|a, b| a.filename.cmp(&b.filename));

    Ok(models)
}

#[tauri::command]
async fn delete_model_file(app: tauri::AppHandle, filename: String) -> Result<bool, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let models_dir = app_data_dir.join("binaries").join("models");
    let model_path = models_dir.join(&filename);

    // Validate the filename to prevent directory traversal attacks
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err("Invalid filename".to_string());
    }

    // Check if file exists
    if !model_path.exists() {
        return Err(format!("Model file not found: {}", filename));
    }

    // Delete the file
    fs::remove_file(&model_path).map_err(|e| format!("Failed to delete model file: {}", e))?;

    println!("Model file deleted: {}", filename);
    Ok(true)
}

#[tauri::command]
async fn download_custom_model(
    app: tauri::AppHandle,
    url: String,
    filename: String,
) -> Result<String, String> {
    println!("Downloading custom model from {} to {}", url, filename);

    // Validate filename
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err("Invalid filename".to_string());
    }

    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    // Ensure directories exist
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }

    // Create ModelDownloadInfo for the download using the create_custom_model_info function
    let model_info = create_custom_model_info(
        format!("Custom model: {}", filename),
        url.clone(),
        filename.clone(),
        0.0, // Unknown size for custom downloads
    );

    // Use existing download_model function
    match download_model(&app, model_info).await {
        Ok(path) => {
            println!("Custom model downloaded successfully to: {:?}", path);
            Ok(path.to_string_lossy().to_string())
        }
        Err(e) => Err(format!("Download failed: {}", e)),
    }
}

/// Get all available Whisper model options with metadata
#[tauri::command]
async fn get_whisper_model_options_command() -> Result<Vec<WhisperModelMetadata>, String> {
    Ok(get_whisper_model_options())
}

/// Get runtime binaries metadata
#[tauri::command]
async fn get_runtime_binaries_command() -> Result<Vec<RuntimeBinaryMetadata>, String> {
    Ok(get_runtime_binaries())
}

/// Get MedLlama model metadata
#[tauri::command]
async fn get_medllama_metadata_command() -> Result<MedLlamaModelMetadata, String> {
    Ok(get_medllama_metadata())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            ensure_app_directory,
            validate_audio_file,
            transcribe_audio,
            generate_medical_note,
            create_patient_note,
            load_patient_notes,
            update_patient_note,
            delete_patient_note,
            delete_audio_file,
            check_auth_status,
            create_user_account_command,
            authenticate_user_command,
            get_user_info_command,
            check_setup_status,
            get_required_models_list,
            check_models_downloaded,
            check_all_models_installed,
            get_models_info_command,
            download_model_file,
            complete_setup,
            get_model_preferences_command,
            save_model_preferences_command,
            list_downloaded_models,
            delete_model_file,
            download_custom_model,
            get_whisper_model_options_command,
            get_runtime_binaries_command,
            get_medllama_metadata_command
        ])
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("failed to get resource directory");
            println!("Resource directory: {:?}", resource_dir);

            // Create app local data directory if it doesn't exist
            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("failed to get app local data directory");
            if !app_data_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                    println!("Failed to create app data directory: {}", e);
                } else {
                    println!("Created app data directory: {:?}", app_data_dir);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
