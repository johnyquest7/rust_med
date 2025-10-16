use crate::db::ModelPreferences;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid URL: {0}")]
    #[allow(dead_code)]
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

/// Metadata about a Whisper model option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModelMetadata {
    pub value: String,
    pub label: String,
    pub size: f64,
    pub url: String,
    pub file_name: String,
}

/// Metadata about the fixed runtime binaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBinaryMetadata {
    pub name: String,
    pub url: String,
    pub file_name: String,
    pub size_mb: f64,
}

/// Metadata about the default MedLlama model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedLlamaModelMetadata {
    pub name: String,
    pub default_url: String,
    pub file_name: String,
    pub size_mb: f64,
}

/// Get all available Whisper model options with metadata
/// This is the SINGLE SOURCE OF TRUTH for Whisper model information
pub fn get_whisper_model_options() -> Vec<WhisperModelMetadata> {
    vec![
        WhisperModelMetadata {
            value: "tiny".to_string(),
            label: "Tiny (141 MB) - Fastest".to_string(),
            size: 141.0,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
                .to_string(),
            file_name: "whisper-tiny.en.gguf".to_string(),
        },
        WhisperModelMetadata {
            value: "base".to_string(),
            label: "Base (142 MB) - Fast".to_string(),
            size: 142.0,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
                .to_string(),
            file_name: "whisper-base.en.gguf".to_string(),
        },
        WhisperModelMetadata {
            value: "small".to_string(),
            label: "Small (466 MB) - Balanced".to_string(),
            size: 466.0,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin"
                .to_string(),
            file_name: "whisper-small.en.gguf".to_string(),
        },
        WhisperModelMetadata {
            value: "medium".to_string(),
            label: "Medium (1.5 GB) - Accurate".to_string(),
            size: 1500.0,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin"
                .to_string(),
            file_name: "whisper-medium.en.gguf".to_string(),
        },
        WhisperModelMetadata {
            value: "large".to_string(),
            label: "Large (3.1 GB) - Most Accurate".to_string(),
            size: 3100.0,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .to_string(),
            file_name: "whisper-large.gguf".to_string(),
        },
    ]
}

/// Get runtime binaries metadata
/// This is the SINGLE SOURCE OF TRUTH for runtime binary information
pub fn get_runtime_binaries() -> Vec<RuntimeBinaryMetadata> {
    vec![
        RuntimeBinaryMetadata {
            name: "Whisperfile (Transcription Engine)".to_string(),
            url:
                "https://huggingface.co/Mozilla/whisperfile/resolve/main/whisper-tiny.en.llamafile"
                    .to_string(),
            file_name: "whisperfile".to_string(),
            size_mb: 83.0,
        },
        RuntimeBinaryMetadata {
            name: "Llamafile (LLM Runtime)".to_string(),
            url:
                "https://github.com/Mozilla-Ocho/llamafile/releases/download/0.9.3/llamafile-0.9.3"
                    .to_string(),
            file_name: "llamafile".to_string(),
            size_mb: 293.0,
        },
    ]
}

/// Get default MedLlama model metadata
/// This is the SINGLE SOURCE OF TRUTH for MedLlama model information
pub fn get_medllama_metadata() -> MedLlamaModelMetadata {
    MedLlamaModelMetadata {
        name: "MedLlama Model (Medical Notes)".to_string(),
        default_url: "https://huggingface.co/Johnyquest7/med_llm_small/resolve/main/med_llama.gguf"
            .to_string(),
        file_name: "med_llama.gguf".to_string(),
        size_mb: 770.0,
    }
}

/// Get whisper model info based on model size
pub fn get_whisper_model_info(size: WhisperModelSize) -> ModelDownloadInfo {
    match size {
        WhisperModelSize::Tiny => ModelDownloadInfo {
            name: "Whisper Tiny Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
                .to_string(),
            file_name: "whisper-tiny.en.gguf".to_string(),
            size_mb: 141.0,
        },
        WhisperModelSize::Base => ModelDownloadInfo {
            name: "Whisper Base Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
                .to_string(),
            file_name: "whisper-base.en.gguf".to_string(),
            size_mb: 142.0,
        },
        WhisperModelSize::Small => ModelDownloadInfo {
            name: "Whisper Small Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin"
                .to_string(),
            file_name: "whisper-small.en.gguf".to_string(),
            size_mb: 466.0,
        },
        WhisperModelSize::Medium => ModelDownloadInfo {
            name: "Whisper Medium Model (English)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin"
                .to_string(),
            file_name: "whisper-medium.en.gguf".to_string(),
            size_mb: 1500.0,
        },
        WhisperModelSize::Large => ModelDownloadInfo {
            name: "Whisper Large Model (Multilingual)".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .to_string(),
            file_name: "whisper-large.gguf".to_string(),
            size_mb: 3100.0,
        },
    }
}

/// Create ModelDownloadInfo from custom URL and model name
pub fn create_custom_model_info(
    name: String,
    url: String,
    file_name: String,
    size_mb: f64,
) -> ModelDownloadInfo {
    ModelDownloadInfo {
        name,
        url,
        file_name,
        size_mb,
    }
}

/// Get the list of models that need to be downloaded (uses default preferences)
pub fn get_required_models() -> Vec<ModelDownloadInfo> {
    let mut models = Vec::new();

    // Add runtime binaries
    for binary in get_runtime_binaries() {
        models.push(ModelDownloadInfo {
            name: binary.name,
            url: binary.url,
            file_name: binary.file_name,
            size_mb: binary.size_mb,
        });
    }

    // Add default Whisper model (tiny)
    models.push(get_whisper_model_info(WhisperModelSize::Tiny));

    // Add default MedLlama model
    let medllama = get_medllama_metadata();
    models.push(ModelDownloadInfo {
        name: medllama.name,
        url: medllama.default_url,
        file_name: medllama.file_name,
        size_mb: medllama.size_mb,
    });

    models
}

/// Get the list of models that need to be downloaded based on user preferences
pub fn get_required_models_with_preferences(
    preferences: &ModelPreferences,
) -> Vec<ModelDownloadInfo> {
    let mut models = Vec::new();

    // Add runtime binaries
    for binary in get_runtime_binaries() {
        models.push(ModelDownloadInfo {
            name: binary.name,
            url: binary.url,
            file_name: binary.file_name,
            size_mb: binary.size_mb,
        });
    }

    // Parse the whisper model size from preferences
    let whisper_size = match preferences.whisper_model_size.as_str() {
        "tiny" => WhisperModelSize::Tiny,
        "base" => WhisperModelSize::Base,
        "small" => WhisperModelSize::Small,
        "medium" => WhisperModelSize::Medium,
        "large" => WhisperModelSize::Large,
        _ => WhisperModelSize::Tiny, // Default fallback
    };
    models.push(get_whisper_model_info(whisper_size));

    // Add MedLlama model (uses user preference URL or default)
    let medllama = get_medllama_metadata();
    models.push(ModelDownloadInfo {
        name: medllama.name,
        url: preferences.med_llama_url.clone(),
        file_name: preferences.med_llama_filename.clone(),
        size_mb: medllama.size_mb,
    });

    models
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

/// Check if all required models are already downloaded based on user preferences
pub async fn check_models_exist_with_preferences(
    app: &AppHandle,
    preferences: &ModelPreferences,
) -> Result<Vec<(ModelDownloadInfo, bool)>, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    let models = get_required_models_with_preferences(preferences);
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
#[allow(dead_code)]
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

/// Get detailed information about all models including their installation status based on user preferences
pub async fn get_models_info_with_preferences(
    app: &AppHandle,
    preferences: &ModelPreferences,
) -> Result<Vec<ModelInfo>, String> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let binaries_dir = app_data_dir.join("binaries");
    let models_dir = binaries_dir.join("models");

    let models = get_required_models_with_preferences(preferences);
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
#[allow(dead_code)]
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
                if file_name.starts_with("whisper-")
                    && (file_name.ends_with(".gguf") || file_name.ends_with(".bin"))
                {
                    whisper_models.push(file_name);
                }
            }
        }
    }

    whisper_models.sort();
    Ok(whisper_models)
}

/// Delete a model file by filename
#[allow(dead_code)]
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

    std::fs::remove_file(&file_path).map_err(|e| format!("Failed to delete model file: {}", e))?;

    Ok(())
}

/// Download a single model file with progress tracking
pub async fn download_model(
    app: &AppHandle,
    model: ModelDownloadInfo,
) -> Result<PathBuf, DownloadError> {
    let app_data_dir = app
        .path()
        .app_local_data_dir()
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
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                file_name: model.file_name.clone(),
                downloaded_bytes: 0,
                total_bytes: Some((model.size_mb * 1024.0 * 1024.0) as u64),
                percentage: 100.0,
                status: DownloadStatus::Completed,
            },
        );
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
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            file_name: model.file_name.clone(),
            downloaded_bytes: 0,
            total_bytes: total_size,
            percentage: 0.0,
            status: DownloadStatus::Downloading,
        },
    );

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
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    file_name: model.file_name.clone(),
                    downloaded_bytes: downloaded,
                    total_bytes: total_size,
                    percentage,
                    status: DownloadStatus::Downloading,
                },
            );
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
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            file_name: model.file_name.clone(),
            downloaded_bytes: downloaded,
            total_bytes: total_size,
            percentage: 100.0,
            status: DownloadStatus::Completed,
        },
    );

    println!("Download complete: {:?}", target_path);

    Ok(target_path)
}
