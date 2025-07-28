use tauri::{command, AppHandle, State, Manager};
use tauri_plugin_mic_recorder::{MicRecorderExt, RecordingOptions};
use tauri_plugin_shell::ShellExt;
use std::sync::Mutex;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct AppState {
    pub recording_state: Mutex<bool>,
}

#[derive(Serialize)]
pub struct RecordingState {
    is_recording: bool,
}

#[derive(Serialize)]
pub struct TranscriptionResult {
    success: bool,
    transcript: String,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct MedicalNoteResult {
    success: bool,
    note: String,
    error: Option<String>,
}

#[command]
pub async fn start_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut recording = state.recording_state.lock().unwrap();
    
    if *recording {
        return Err("Already recording".into());
    }

    let options = RecordingOptions {
        sample_rate: Some(16000),
        channels: Some(1),
        format: Some("wav".to_string()),
    };

    match app.mic_recorder().start_recording("output.wav".into(), Some(options)).await {
        Ok(_) => {
            *recording = true;
            // Emit event to frontend
            app.emit("recording-state-changed", RecordingState { is_recording: true })
                .map_err(|e| format!("Failed to emit event: {}", e))?;
            Ok("Recording started".into())
        }
        Err(e) => Err(format!("Failed to start recording: {}", e)),
    }
}

#[command]
pub async fn stop_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut recording = state.recording_state.lock().unwrap();
    
    if !*recording {
        return Err("Not currently recording".into());
    }

    match app.mic_recorder().stop_recording().await {
        Ok(_) => {
            *recording = false;
            // Emit event to frontend
            app.emit("recording-state-changed", RecordingState { is_recording: false })
                .map_err(|e| format!("Failed to emit event: {}", e))?;
            Ok("Recording stopped".into())
        }
        Err(e) => Err(format!("Failed to stop recording: {}", e)),
    }
}

#[command]
pub async fn transcribe_audio(app: AppHandle) -> Result<TranscriptionResult, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    
    // Determine the correct whisperfile executable
    let whisperfile_name = if cfg!(target_os = "windows") {
        "whisperfile.exe"
    } else {
        "whisperfile"
    };
    
    let whisperfile_path = resource_dir.join("binaries").join(whisperfile_name);
    let model_path = resource_dir.join("binaries").join("models").join("whisper-tiny.en.gguf");
    let audio_path = app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("output.wav");

    // Execute whisperfile as sidecar
    let output = app
        .shell()
        .command(whisperfile_path)
        .args([
            "-m", &model_path.to_string_lossy(),
            "-f", &audio_path.to_string_lossy(),
            "--no-prints"
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute whisperfile: {}", e))?;

    if output.status.success() {
        let transcript = parse_whisper_output(&output.stdout);
        Ok(TranscriptionResult {
            success: true,
            transcript,
            error: None,
        })
    } else {
        Ok(TranscriptionResult {
            success: false,
            transcript: String::new(),
            error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

#[command]
pub async fn generate_medical_note(
    app: AppHandle,
    transcript: String,
) -> Result<MedicalNoteResult, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    
    // Determine the correct llamafile executable
    let llamafile_name = if cfg!(target_os = "windows") {
        "llamafile.exe"
    } else {
        "llamafile"
    };
    
    let llamafile_path = resource_dir.join("binaries").join(llamafile_name);
    let model_path = resource_dir.join("binaries").join("models").join("med_llama.gguf");

    let prompt = format!(
        "Given the following doctor-patient conversation transcript, create a concise medical note in SOAP format:\n\nTRANSCRIPT:\n{}\n\nPlease provide a well-structured medical note with:\n- Subjective: Patient's chief complaint and symptoms\n- Objective: Observable findings and vital signs if mentioned\n- Assessment: Clinical impression or diagnosis\n- Plan: Treatment plan and follow-up instructions\n\nMEDICAL NOTE:",
        transcript
    );

    // Execute llamafile as sidecar
    let output = app
        .shell()
        .command(llamafile_path)
        .args([
            "-m", &model_path.to_string_lossy(),
            "--temp", "0.3",
            "--no-display-prompt",
            "-p", &prompt
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute llamafile: {}", e))?;

    if output.status.success() {
        let note = output.stdout.trim().to_string();
        Ok(MedicalNoteResult {
            success: true,
            note,
            error: None,
        })
    } else {
        Ok(MedicalNoteResult {
            success: false,
            note: String::new(),
            error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

#[command]
pub async fn get_recording_state(state: State<'_, AppState>) -> Result<RecordingState, String> {
    let recording = state.recording_state.lock().unwrap();
    Ok(RecordingState {
        is_recording: *recording,
    })
}

fn parse_whisper_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut transcript_parts = Vec::new();
    
    for line in lines {
        // Whisper output format: [00:00:00.000 --> 00:00:05.000] Transcript text
        if let Some(bracket_end) = line.find("] ") {
            if line.starts_with('[') {
                let text_part = &line[bracket_end + 2..];
                if !text_part.trim().is_empty() {
                    transcript_parts.push(text_part.trim());
                }
            }
        } else if !line.trim().is_empty() && !line.starts_with('[') {
            // Handle lines without timestamps
            transcript_parts.push(line.trim());
        }
    }
    
    transcript_parts.join(" ")

}