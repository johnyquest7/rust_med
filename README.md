# Medical Note Generator

A desktop application built with Tauri that generates medical notes using on-device AI models for transcription and medical text generation.

## Features

- **Audio Recording**: Record audio directly in the application
- **Speech-to-Text**: Transcribe audio using Whisper models
- **Medical Note Generation**: Generate medical notes using Llama models
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Privacy-First**: All processing happens locally on your device

## System Requirements

- **Operating System**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 18.04+)
- **RAM**: Minimum 8GB, Recommended 16GB+
- **Storage**: At least 5GB free space for models and application
- **CPU**: Modern multi-core processor (Intel i5/AMD Ryzen 5 or better)

## Installation

### 1. Install System Dependencies

#### Install Rust

```bash
# Visit https://rustup.rs/ or run:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Restart your terminal after installation
rustup update
```

#### Install Node.js

```bash
# Visit https://nodejs.org/ or use a version manager:
# Using nvm (recommended):
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install --lts
nvm use --lts

# Or install directly from nodejs.org
```

#### Install Tauri CLI

```bash
cargo install tauri-cli
```

#### Platform-Specific Dependencies

**Windows:**

- Install Microsoft Visual Studio C++ Build Tools
- Install WebView2 Runtime

**macOS:**

```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### 2. Set Up the Project

#### Clone and Install Everything

```bash
# Clone the repository
git clone <your-repository-url>
cd rust_med

# Install all dependencies (this will also download required models)
npm run setup
```

**What happens during setup:**

- Installs npm dependencies
- Downloads Whisper and Llama models automatically
- Downloads platform-specific binaries
- Verifies Rust dependencies
- Creates necessary directories

#### Manual Setup (if npm run setup fails)

If the automatic setup doesn't work, you can run these commands manually:

```bash
# Install npm dependencies
npm install

# Verify Rust dependencies
cargo check

# Create models directory and download models
mkdir -p binaries/models
cd binaries/models

# Download Whisper model
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin -O whisper-tiny.en.gguf

# Download medical Llama model (you'll need to obtain this from a trusted source)
# Place it as med_llama.gguf in the binaries/models/ directory

# Download platform-specific binaries to the binaries/ directory:
# - Windows: whisperfile.exe, llamafile.exe
# - macOS: whisperfile, llamafile
# - Linux: whisperfile, llamafile
```

## Development

### Start Development Server

```bash
npm run dev
```

This will:

- Start the Tauri development server
- Open the application in development mode
- Enable hot reloading for both frontend and backend changes

### Build for Production

```bash
npm run build
```

This creates platform-specific installers in the `src-tauri/target/release/bundle/` directory.

## Project Structure

```
rust_med/
├── src/                    # Frontend web application
│   ├── index.html         # Main HTML file
│   ├── main.js            # Main JavaScript entry point
│   ├── styles.css         # Application styles
│   └── modules/           # JavaScript modules
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Main Rust entry point
│   │   └── commands.rs    # Tauri commands
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── binaries/               # External binaries and models
│   └── models/            # AI model files
└── package.json           # Node.js dependencies
```

## Troubleshooting

### Common Issues

**Rust Build Errors:**

```bash
# Update Rust toolchain
rustup update
# Clean and rebuild
cargo clean
cargo build
```

**Node.js Version Issues:**

```bash
# Check Node.js version (requires 18+)
node --version
# Use correct version with nvm
nvm use --lts
```

**Missing System Dependencies:**

- Windows: Ensure Visual Studio Build Tools are installed
- macOS: Run `xcode-select --install`
- Linux: Install required packages with `apt` or `yum`

**Model Loading Issues:**

- Verify model files are in `binaries/models/`
- Check file permissions
- Ensure sufficient RAM for model loading

### Getting Help

- Check the [Tauri documentation](https://tauri.app/docs)
- Review [Rust documentation](https://doc.rust-lang.org/)
- Open an issue in the project repository

install cargo
cargo install tauri-cli
