use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use std::io::Write;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Download failed: {0}")]
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub file_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Downloading,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadInfo {
    pub name: String,
    pub url: String,
    pub file_name: String,
    pub size_mb: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhisperModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

/// Get whisper model info based on model size
pub fn get_whisper_model_info(size: WhisperModelSize) -> ModelDownloadInfo {
    match size {
        WhisperModelSize::Tiny => ModelDownloadInfo {
            name: "Whisper Tiny Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin".to_string(),
            file_name: "whisper-tiny.en.gguf".to_string(),
            size_mb: 141.0,
        },
        WhisperModelSize::Base => ModelDownloadInfo {
            name: "Whisper Base Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin".to_string(),
            file_name: "whisper-base.en.gguf".to_string(),
            size_mb: 142.0,
        },
        WhisperModelSize::Small => ModelDownloadInfo {
            name: "Whisper Small Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin".to_string(),
            file_name: "whisper-small.en.gguf".to_string(),
            size_mb: 466.0,
        },
        WhisperModelSize::Medium => ModelDownloadInfo {
            name: "Whisper Medium Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin".to_string(),
            file_name: "whisper-medium.en.gguf".to_string(),
            size_mb: 1500.0,
        },
        WhisperModelSize::Large => ModelDownloadInfo {
            name: "Whisper Large Model (Multilingual)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
            file_name: "whisper-large.gguf".to_string(),
            size_mb: 3100.0,
        },
    }
}

/// Create ModelDownloadInfo from custom URL and model name
pub fn create_custom_model_info(name: String, url: String, file_name: String, size_mb: f64) -> ModelDownloadInfo {
    ModelDownloadInfo {
        name,
        url,
        file_name,
        size_mb,
    }
}

/// Get the list of models that need to be downloaded (uses default preferences)
pub fn get_required_models() -> Vec<ModelDownloadInfo> {
    vec![
        ModelDownloadInfo {
            name: "Whisperfile (Transcription Engine)".to_string(),
            url: "https://huggingface.co/Mozilla/whisperfile/resolve/main/whisper-tiny.en.llamafile".to_string(),
            file_name: "whisperfile".to_string(),
            size_mb: 83.0,
        },
        ModelDownloadInfo {
            name: "Llamafile (LLM Runtime)".to_string(),
            url: "https://github.com/Mozilla-Ocho/llamafile/releases/download/0.9.3/llamafile-0.9.3".to_string(),
            file_name: "llamafile".to_string(),
            size_mb: 293.0,
        },
        get_whisper_model_info(WhisperModelSize::Tiny),
        ModelDownloadInfo {
            name: "MedLlama Model (Medical Notes)".to_string(),
            url: "https://huggingface.co/Johnyquest7/med_llm_small/resolve/main/med_llama.gguf".to_string(),
            file_name: "med_llama.gguf".to_string(),
            size_mb: 807.7,
        },
    ]
}

/// Check if all required models are already downloaded
pub async fn check_models_exist(app: &AppHandle) -> Result<Vec<(ModelDownloadInfo, bool)>, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    let models = get_required_models();
    let mut results = Vec::new();

    for model in models {
        let path = if model.file_name.ends_with(".gguf") {
            models_dir.join(&model.file_name)
        } else {
            binaries_dir.join(&model.file_name)
        };

        let exists = path.exists();
        results.push((model, exists));
    }

    Ok(results)
}

/// Check if all required models are present (returns true only if ALL models exist)
pub async fn check_all_models_present(app: &AppHandle) -> Result<bool, String> {
    let model_statuses = check_models_exist(app).await?;

    // Return true only if all models exist
    let all_present = model_statuses.iter().all(|(_, exists)| *exists);

    Ok(all_present)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub file_name: String,
    pub size_mb: f64,
    pub installed: bool,
    pub file_path: Option<String>,
}

/// Get detailed information about all models including their installation status
pub async fn get_models_info(app: &AppHandle) -> Result<Vec<ModelInfo>, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    let models = get_required_models();
    let mut results = Vec::new();

    for model in models {
        let path = if model.file_name.ends_with(".gguf") {
            models_dir.join(&model.file_name)
        } else {
            binaries_dir.join(&model.file_name)
        };

        let installed = path.exists();
        let file_path = if installed {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        };

        results.push(ModelInfo {
            name: model.name,
            file_name: model.file_name,
            size_mb: model.size_mb,
            installed,
            file_path,
        });
    }

    Ok(results)
}

/// List all downloaded whisper models in the models directory
pub async fn list_downloaded_whisper_models(app: &AppHandle) -> Result<Vec<String>, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let models_dir = app_data_dir.join("binaries").join("models");

    if !models_dir.exists() {
        return Ok(Vec::new());
    }

    let mut whisper_models = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with("whisper-") && (file_name.ends_with(".gguf") || file_name.ends_with(".bin")) {
                    whisper_models.push(file_name);
                }
            }
        }
    }

    whisper_models.sort();
    Ok(whisper_models)
}

/// Delete a model file by filename
pub async fn delete_model_file(app: &AppHandle, file_name: String) -> Result<(), String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    // Determine path based on file extension
    let file_path = if file_name.ends_with(".gguf") || file_name.ends_with(".bin") {
        models_dir.join(&file_name)
    } else {
        binaries_dir.join(&file_name)
    };

    if !file_path.exists() {
        return Err(format!("Model file not found: {}", file_name));
    }

    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete model file: {}", e))?;

    Ok(())
}

/// Download a single model file with progress tracking
pub async fn download_model(
    app: &AppHandle,
    model: ModelDownloadInfo,
) -> Result<PathBuf, DownloadError> {
    let app_data_dir = app.path().app_local_data_dir()
        .map_err(|e| DownloadError::Failed(e.to_string()))?;

    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    // Create directories
    std::fs::create_dir_all(&binaries_dir)?;
    std::fs::create_dir_all(&models_dir)?;

    // Determine target path
    let target_path = if model.file_name.ends_with(".gguf") {
        models_dir.join(&model.file_name)
    } else {
        binaries_dir.join(&model.file_name)
    };

    // If file already exists, skip download
    if target_path.exists() {
        let _ = app.emit("download-progress", DownloadProgress {
            file_name: model.name.clone(),
            downloaded_bytes: 0,
            total_bytes: Some((model.size_mb * 1024.0 * 1024.0) as u64),
            percentage: 100.0,
            status: DownloadStatus::Completed,
        });
        return Ok(target_path);
    }

    println!("Downloading {} from {}", model.name, model.url);

    // Start download
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout
        .build()?;

    let response = client.get(&model.url).send().await?;

    if !response.status().is_success() {
        return Err(DownloadError::Failed(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let total_size = response.content_length();

    // Create temporary file
    let temp_path = target_path.with_extension("tmp");
    let mut file = std::fs::File::create(&temp_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    // Emit initial progress
    let _ = app.emit("download-progress", DownloadProgress {
        file_name: model.name.clone(),
        downloaded_bytes: 0,
        total_bytes: total_size,
        percentage: 0.0,
        status: DownloadStatus::Downloading,
    });

    // Download in chunks and emit progress
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        let percentage = if let Some(total) = total_size {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Emit progress every 1MB or so to avoid overwhelming the frontend
        if downloaded % (1024 * 1024) < 8192 || percentage >= 100.0 {
            let _ = app.emit("download-progress", DownloadProgress {
                file_name: model.name.clone(),
                downloaded_bytes: downloaded,
                total_bytes: total_size,
                percentage,
                status: DownloadStatus::Downloading,
            });
        }
    }

    // Finalize download
    file.flush()?;
    drop(file);

    // Rename temp file to final name
    std::fs::rename(&temp_path, &target_path)?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        if !model.file_name.ends_with(".gguf") {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&target_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&target_path, perms)?;
        }
    }

    // Emit completion
    let _ = app.emit("download-progress", DownloadProgress {
        file_name: model.name.clone(),
        downloaded_bytes: downloaded,
        total_bytes: total_size,
        percentage: 100.0,
        status: DownloadStatus::Completed,
    });

    println!("Download complete: {:?}", target_path);

    Ok(target_path)
}
