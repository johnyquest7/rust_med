# Medical Note Generator Setup Instructions

## Files to Update

After running this setup script, you need to copy the code from the Tauri implementation artifact into these files:

### Backend Files (Rust)
- `src-tauri/src/main.rs` - Copy main.rs code from artifact
- `src-tauri/src/commands.rs` - Copy commands.rs code from artifact
- `src-tauri/tauri.conf.json` - Copy tauri.conf.json code from artifact
- `src-tauri/capabilities/default.json` - Copy default.json code from artifact

### Frontend Files (Web)
- `src/index.html` - Copy index.html code from artifact
- `src/styles.css` - Copy styles.css code from artifact
- `src/main.js` - Copy main.js code from artifact

## Required Binaries

Place these files in the `binaries/` directory:
- `whisperfile.exe` (Windows) or `whisperfile` (Linux/macOS)
- `llamafile.exe` (Windows) or `llamafile` (Linux/macOS)

Place these model files in the `binaries/models/` directory:
- `whisper-tiny.en.gguf`
- `med_llama.gguf`

## Prerequisites

1. Install Rust: https://rustup.rs/
2. Install Node.js: https://nodejs.org/
3. Install Tauri CLI: `cargo install tauri-cli`

## Setup Commands

```bash
# Install npm dependencies
npm install

# Development mode
npm run dev

# Build for production
npm run build
```

## Icon Files

You'll need to add icon files to `src-tauri/icons/`:
- 32x32.png
- 128x128.png
- 128x128@2x.png
- icon.icns (macOS)
- icon.ico (Windows)
