# Medical Note Generator - Distribution Guide

## For Users

### Installation

1. Download the installer for your platform:
   - **macOS**: `Medical Note Generator.dmg`
   - **Windows**: `Medical Note Generator.msi`
   - **Linux**: `medical-note-generator.deb` or `medical-note-generator.AppImage`

2. Run the installer and follow the prompts

3. Launch the application

### First-Time Setup

When you first open Medical Note Generator, you'll see a **Setup Wizard** that guides you through downloading the required AI models.

#### What Gets Downloaded

The application downloads 4 AI models (~4.1 GB total):

1. **Whisperfile** (83 MB) - Speech-to-text transcription engine
2. **Llamafile** (293 MB) - LLM runtime for processing
3. **Whisper Tiny Model** (141 MB) - English speech recognition model
4. **MedLlama Model** (3.6 GB) - Medical note generation model

#### Setup Process

1. The setup wizard will appear automatically on first launch
2. Click **"Start Download"** to begin
3. Watch the progress bars as each model downloads
4. Once complete, click **"Continue to App"**
5. Create your account and start using the application

**Note:** All models are stored locally in your application data directory:
- **macOS**: `~/Library/Application Support/com.medical.notegenerator/binaries/`
- **Windows**: `%APPDATA%\com.medical.notegenerator\binaries\`
- **Linux**: `~/.local/share/com.medical.notegenerator/binaries/`

### Privacy & Local Processing

All data processing happens **entirely on your device**:
- Audio recordings are transcribed locally
- Medical notes are generated locally
- Patient data is encrypted and stored locally
- No data is ever sent to external servers

---

## For Developers

### Building the Distribution

To create a distributable installer:

```bash
# Install dependencies
npm install
cd frontend && npm install && cd ..

# Build for production
npm run build
# or
cargo tauri build
```

The installer will be created in `src-tauri/target/release/bundle/`:
- macOS: `dmg/Medical Note Generator_0.1.0_*.dmg`
- Windows: `msi/Medical Note Generator_0.1.0_*.msi`
- Linux: `deb/`, `appimage/`, or `rpm/`

### Key Implementation Details

#### Setup Wizard Flow

1. **Check Setup Status** (`check_setup_status`)
   - On app launch, checks if models have been downloaded
   - Stored in SQLite database (`setup_status` table)

2. **Display Setup Wizard** (`SetupWizard.svelte`)
   - Shows if `setup_completed = false`
   - Lists all required models with download progress

3. **Download Models** (`download_model_file`)
   - Downloads each model sequentially
   - Emits progress events (`download-progress`)
   - Saves to `{app_local_data_dir}/binaries/`

4. **Mark Complete** (`complete_setup`)
   - Updates database when all downloads finish
   - Redirects to authentication/main app

#### Binary Path Resolution

The application looks for binaries in this order:

1. **Production** (after setup wizard): `{app_local_data_dir}/binaries/`
2. **Development**: `./binaries/` (project root)

This allows development without running the setup wizard.

### Customization

#### Changing Model URLs

Edit `src-tauri/src/downloads.rs`, function `get_required_models()`:

```rust
pub fn get_required_models() -> Vec<ModelDownloadInfo> {
    vec![
        ModelDownloadInfo {
            name: "Model Name".to_string(),
            url: "https://example.com/model.file".to_string(),
            file_name: "local_filename".to_string(),
            size_mb: 100.0,
        },
        // ... more models
    ]
}
```

#### Skipping Setup Wizard (Development)

1. Place binaries manually in `binaries/` directory:
   ```
   binaries/
   ├── whisperfile
   ├── llamafile
   └── models/
       ├── whisper-tiny.en.gguf
       └── med_llama.gguf
   ```

2. Or mark setup as complete in database:
   ```sql
   INSERT INTO setup_status (id, setup_completed, completed_at)
   VALUES (1, 1, datetime('now'));
   ```

### Architecture Changes

**Before:**
- Models bundled with app (~4.1 GB installer)
- Models in Tauri resources directory

**After:**
- Lightweight installer (~10-20 MB)
- Setup wizard downloads models on first run
- Models stored in app local data directory
- Better user experience for distribution

### Testing

To test the setup wizard:

1. Delete the database and binaries:
   ```bash
   rm -rf ~/Library/Application\ Support/com.medical.notegenerator/
   ```

2. Launch the app - setup wizard should appear

3. Click through the download process

4. Verify app works after setup completes
