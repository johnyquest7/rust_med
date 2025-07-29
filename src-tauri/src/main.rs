#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{Manager, Emitter};
use serde::{Serialize, Deserialize};
use tauri_plugin_shell::ShellExt;
use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Local};
use std::io::{BufRead, BufReader};
use std::process::Stdio;

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
    dob: String,
    note_type: String,
    transcript: String,
    medical_note: String,
    created_at: DateTime<Local>,
}

#[derive(Serialize)]
struct SaveNoteResult {
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
        std::fs::create_dir_all(&app_data_dir).map_err(|e| {
            format!("Failed to create app directory: {}", e)
        })?;
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
async fn transcribe_audio(app: tauri::AppHandle, audio_path: String) -> Result<TranscriptionResult, String> {
    println!("Starting transcription for: {}", audio_path);
    
    // Emit transcription started event
    app.emit("transcription-progress", "Starting transcription...").ok();
    
    // Validate audio file first
    if let Err(validation_error) = validate_audio_file(audio_path.clone()).await {
        return Ok(TranscriptionResult {
            success: false,
            transcript: String::new(),
            error: Some(format!("Audio validation failed: {}", validation_error)),
        });
    }
    
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    println!("Resource directory: {:?}", resource_dir);
    
    // Determine the correct whisperfile executable
    let whisperfile_name = if cfg!(target_os = "windows") {
        "whisperfile.exe"
    } else {
        "whisperfile"
    };
    
    // Try different possible locations for the whisperfile
    let whisperfile_paths = [
        // Development paths (relative to project root)
        PathBuf::from("binaries").join(whisperfile_name),
        PathBuf::from("./binaries").join(whisperfile_name),
        PathBuf::from("../binaries").join(whisperfile_name),
        // Production paths (in resources)
        resource_dir.join(whisperfile_name),
        resource_dir.join("binaries").join(whisperfile_name),
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
                error: Some(format!("Whisperfile not found. Tried: {:?}", whisperfile_paths)),
            });
        }
    };

    // Find the model file
    let model_names = [
        "whisper-tiny.en.gguf",
        "ggml-tiny.en.bin", 
        "whisper-tiny.en.bin",
        "whisper-small.en.gguf",
        "ggml-small.en.bin"
    ];
    
    let model_paths = [
        // Development paths
        PathBuf::from("binaries").join("models"),
        PathBuf::from("./binaries").join("models"),
        PathBuf::from("../binaries").join("models"),
        // Production paths
        resource_dir.join("models"),
        resource_dir.join("binaries").join("models"),
    ];
    
    let mut model_path = None;
    'outer: for base_path in &model_paths {
        for model_name in &model_names {
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
    let is_supported_format = audio_path.ends_with(".wav") || 
                             audio_path.ends_with(".mp3") || 
                             audio_path.ends_with(".flac") || 
                             audio_path.ends_with(".ogg");
    
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
    println!("Executing whisperfile with args: -m {:?} -f {} --no-prints", model_path, audio_path);
    
    // Emit progress update
    app.emit("transcription-progress", "Processing audio with Whisper model...").ok();
    
    let output = app
        .shell()
        .command(&whisperfile_path)
        .args([
            "-m", &model_path.to_string_lossy(),
            "-f", &audio_path,
            "--no-prints"  // Suppress debug output - this is the key flag for whisperfile
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
        if stderr_str.contains("failed to read pcm frames") || stderr_str.contains("At end otalerror") {
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
    println!("Starting medical note generation for transcript: {}", transcript);
    
    // Emit note generation started event
    let note_type_display = if note_type == "soap" { "SOAP" } else { "Full" };
    app.emit("note-generation-progress", format!("Generating {} medical note...", note_type_display)).ok();
    
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    
    // Determine the correct llamafile executable
    let llamafile_name = if cfg!(target_os = "windows") {
        "llamafile.exe"
    } else {
        "llamafile"
    };
    
    // Try different possible locations for the llamafile
    let llamafile_paths = [
        // Development paths (relative to project root)
        PathBuf::from("binaries").join(llamafile_name),
        PathBuf::from("./binaries").join(llamafile_name),
        PathBuf::from("../binaries").join(llamafile_name),
        // Production paths (in resources)
        resource_dir.join(llamafile_name),
        resource_dir.join("binaries").join(llamafile_name),
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

    // Find the model file with absolute paths - FIXED VERSION
    let model_names = [
        "med_llama.gguf",
        "llama-2-7b-chat.gguf", 
        "llama-2-13b-chat.gguf",
        "mistral-7b-instruct.gguf",
        "openchat-3.5.gguf"
    ];
    
    // Get the current working directory to build absolute paths
    let current_dir = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let project_root = if current_dir.ends_with("src-tauri") {
        current_dir.parent().unwrap_or(&current_dir).to_path_buf()
    } else {
        current_dir
    };
    
    let model_paths = [
        // Absolute paths from project root
        project_root.join("binaries").join("models"),
        project_root.join("binaries"),  // In case models are directly in binaries
        // Try resource dir paths
        resource_dir.join("binaries").join("models"),
        resource_dir.join("models"),
    ];
    
    let mut model_path = None;
    'outer_llm: for base_path in &model_paths {
        for model_name in &model_names {
            let test_path = base_path.join(model_name);
            println!("Checking LLM model path: {:?}", test_path);
            if test_path.exists() {
                // Convert to absolute path
                let absolute_path = test_path.canonicalize()
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

    // Use the correct chat template for your model
    let prompt = if note_type == "soap" {
        format!(
            "<|begin_of_text|><|start_header_id|>user<|end_header_id|>
You are an expert medical professor assisting in the creation of medically accurate SOAP notes.  
Create a Medical SOAP note from the transcript, following these guidelines:\n    
Correct any medcal terminology errors that might have happened during transcription before generating the SOAP note.\n
S (Subjective): Summarize the patient's reported symptoms, including chief complaint and relevant history.
Rely on the patient's statements as the primary source and ensure standardized terminology.\n    
O (Objective): Include objective findings in the transcripts such as vital signs, physical exam findings, lab results, and imaging.\n    
A (Assessment): Concise assessment combining subjective and objective data. State the diagnoses and assesment of the diagnoses in a numbered list.\n    
P (Plan): Outline the treatment plan. Compile the report based solely on the transcript provided.\n    
Please format the summary in a clean, simple list format without using markdown or bullet points. Use 'S:', 'O:', 'A:', 'P:' directly followed by the text. TRANSCRIPT: \n

{}
<|eot_id|><|start_header_id|>assistant<|end_header_id|>
S: ",
            transcript
        )
    } else {
        format!(
            "<|begin_of_text|><|start_header_id|>user<|end_header_id|>
You are an expert medical transcriptionist. Correct any medcal terminology errors that might have happened during transcription before generating the medical note. You convert medical transcript to a structured medical note with these sections in this order: 
1. Presenting Illness
(Bullet point statements of the main problem)
2. History of Presenting Illness
(Chronological narrative: symptom onset, progression, modifiers, associated factors)
3. Past Medical History
(List chronic illnesses and past medical diagnoses mentioned in the transcript. Do not include surgeries)
4. Surgical History
(List prior surgeries with year if known mentioned in the transcript)
5. Family History
(Relevant family history mentioned in the transcript)
6. Social History
(Occupation, tobacco/alcohol/drug use, exercise, living situation if mentioned in the transcript)
7. Allergy History
(Drug/food/environmental allergies + reactions - if mentioned in the transcript)
8. Medication History
(List medications the patient is already taking. Do not place any medication the patient is currently not taking.) 
9. Dietary History
(\"Not applicable\" if unrelated, otherwise summarize diet pattern)
10. Review of Systems
(Head-to-toe -ordered bullets; note positives and pertinent negatives- mentioned in the transcript)
11. Physical Exam Findings
Vital Signs (BP, HR, RR, Temp, SpO₂, HT, WT, BMI) - if mentioned in the transcript
(Structured by system: General, HEENT, CV, Resp, Abd, Neuro, MSK, Skin, Psych) - if mentioned in the transcript
12. Labs and Imaging
(labs, imaging results)
13. Assessment and Plan 
(List each diagnoses and treatment plan. No other information needed in this section.Do not generate new diagnoses)

Medical transcript:
{}

<|eot_id|><|start_header_id|>assistant<|end_header_id|>
",
            transcript
        )
    };

    println!("=== PROMPT BEING SENT ===");
    println!("{}", prompt);
    println!("=== END PROMPT ===");
    
    // Execute llamafile with supported parameters only
    println!("Executing llamafile with absolute model path: {:?}", model_path);
    
    let mut cmd = std::process::Command::new(&llamafile_path);
    cmd.current_dir(&project_root)
        .args([
            "-m", &model_path.to_string_lossy(),
            "--temp", "0.2",           // Low temp for consistent output
            "--top-p", "0.95",         
            "--top-k", "30",           
            "--repeat-penalty", "1.05", // Prevent repetition
            "-n", "800",               // Limit output length
            "--threads", "4",
            "--ctx-size", "4096",      
            "--no-display-prompt",     // Don't echo prompt
            "--batch-size", "512",
            "--log-disable",           // Disable logging
            "-p", &prompt
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    let mut child = cmd.spawn()
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
    let status = child.wait().map_err(|e| format!("Failed to wait for llamafile: {}", e))?;

    if status.success() {
        // Clean the final output
        let note = clean_llm_output(&accumulated_output);
        println!("Generated note length: {}", note.len());
        
        if note.trim().is_empty() {
            return Ok(MedicalNoteResult {
                success: false,
                note: String::new(),
                error: Some("LLM produced empty output. Model may have failed to generate response.".to_string()),
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
        } else if !line.starts_with('[') && !line.contains("->") && !line.contains("[BLANK_AUDIO]") {
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
    
    // Since the model starts with "S: " we need to add it back if it was stripped
    let mut cleaned = result.trim().to_string();
    
    // If the output doesn't start with "S:", add it back (only for SOAP notes)
    if !cleaned.starts_with("S:") && !cleaned.is_empty() && 
       !cleaned.contains("Presenting Illness") && !cleaned.contains("History of Presenting Illness") {
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
        if trimmed.starts_with("S:") || trimmed.starts_with("O:") || 
           trimmed.starts_with("A:") || trimmed.starts_with("P:") {
            last_section = &trimmed[0..2];
            final_lines.push(trimmed);
            continue;
        }
        
        // Skip obvious artifacts and repetitive content
        if trimmed.contains("Create a SOAP") || 
           trimmed.contains("medical conversation") ||
           trimmed.contains("Provide only") ||
           trimmed.len() < 3 {
            continue;
        }
        
        // Stop if we see the model continuing the conversation
        if trimmed.contains("Dr. Thomas") && trimmed.contains(":") ||
           trimmed.contains("Susan") && trimmed.contains(":") ||
           trimmed.contains("Patient") && trimmed.contains(":") {
            break;
        }
        
        // Stop if we see obvious model artifacts
        if trimmed.contains("**") || trimmed.contains("###") || 
           trimmed.starts_with("---") || trimmed.contains("```") {
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
async fn save_patient_note(
    app: tauri::AppHandle,
    first_name: String,
    last_name: String,
    dob: String,
    note_type: String,
    transcript: String,
    medical_note: String,
) -> Result<SaveNoteResult, String> {
    println!("Saving patient note for {} {}", first_name, last_name);
    
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let notes_dir = app_data_dir.join("notes");
    
    // Create notes directory if it doesn't exist
    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).map_err(|e| {
            format!("Failed to create notes directory: {}", e)
        })?;
    }
    
    // Generate unique note ID
    let note_id = format!("{}", chrono::Local::now().timestamp_millis());
    let created_at = chrono::Local::now();
    
    let patient_note = PatientNote {
        id: note_id.clone(),
        first_name,
        last_name,
        dob,
        note_type,
        transcript,
        medical_note,
        created_at,
    };
    
    // Save note to JSON file
    let note_file = notes_dir.join(format!("{}.json", note_id));
    let json_content = serde_json::to_string_pretty(&patient_note)
        .map_err(|e| format!("Failed to serialize note: {}", e))?;
    
    fs::write(&note_file, json_content)
        .map_err(|e| format!("Failed to write note file: {}", e))?;
    
    println!("Note saved successfully: {:?}", note_file);
    
    Ok(SaveNoteResult {
        success: true,
        note_id: Some(note_id),
        error: None,
    })
}

#[tauri::command]
async fn load_patient_notes(app: tauri::AppHandle) -> Result<LoadNotesResult, String> {
    println!("Loading patient notes");
    
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let notes_dir = app_data_dir.join("notes");
    
    // Check if notes directory exists
    if !notes_dir.exists() {
        return Ok(LoadNotesResult {
            success: true,
            notes: Vec::new(),
            error: None,
        });
    }
    
    let mut notes = Vec::new();
    
    // Read all JSON files in the notes directory
    match fs::read_dir(&notes_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        // Read and parse the note file
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                match serde_json::from_str::<PatientNote>(&content) {
                                    Ok(note) => notes.push(note),
                                    Err(e) => println!("Failed to parse note file {:?}: {}", path, e),
                                }
                            }
                            Err(e) => println!("Failed to read note file {:?}: {}", path, e),
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Ok(LoadNotesResult {
                success: false,
                notes: Vec::new(),
                error: Some(format!("Failed to read notes directory: {}", e)),
            });
        }
    }
    
    // Sort notes by creation date (newest first)
    notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    println!("Loaded {} notes", notes.len());
    
    Ok(LoadNotesResult {
        success: true,
        notes,
        error: None,
    })
}

#[tauri::command]
async fn delete_patient_note(app: tauri::AppHandle, note_id: String) -> Result<bool, String> {
    println!("Deleting patient note: {}", note_id);
    
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let note_file = app_data_dir.join("notes").join(format!("{}.json", note_id));
    
    if note_file.exists() {
        fs::remove_file(&note_file)
            .map_err(|e| format!("Failed to delete note file: {}", e))?;
        println!("Note deleted successfully");
        Ok(true)
    } else {
        Err(format!("Note file not found: {}", note_id))
    }
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
            save_patient_note,
            load_patient_notes,
            delete_patient_note
        ])
        .setup(|app| {
            let resource_dir = app.path().resource_dir().expect("failed to get resource directory");
            println!("Resource directory: {:?}", resource_dir);
            
            // Create app local data directory if it doesn't exist
            let app_data_dir = app.path().app_local_data_dir().expect("failed to get app local data directory");
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