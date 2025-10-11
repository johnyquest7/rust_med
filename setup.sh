#!/bin/bash

# Medical Note Generator Setup Script
# This script sets up the development environment for the Rust medical application

set -e  # Exit on any error

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# Function to check system requirements
check_system_requirements() {
    echo "Checking system requirements..."
    
    # Check OS
    OS=$(detect_os)
    if [[ "$OS" == "unknown" ]]; then
        echo "ERROR: Unsupported operating system"
        exit 1
    fi
    echo "Detected OS: $OS"
    
    # Check RAM (macOS specific)
    if [[ "$OS" == "macos" ]]; then
        RAM_GB=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
        if [[ $RAM_GB -lt 8 ]]; then
            echo "WARNING: Low RAM detected: ${RAM_GB}GB (minimum 8GB recommended)"
        else
            echo "RAM: ${RAM_GB}GB"
        fi
    fi
    
    # Check available disk space
    DISK_SPACE=$(df . | awk 'NR==2 {print $4}')
    DISK_SPACE_GB=$((DISK_SPACE / 1024 / 1024))
    if [[ $DISK_SPACE_GB -lt 5 ]]; then
        echo "ERROR: Insufficient disk space: ${DISK_SPACE_GB}GB (minimum 5GB required)"
        exit 1
    fi
    echo "Available disk space: ${DISK_SPACE_GB}GB"
}

# Function to install Rust
install_rust() {
    if command_exists rustc && command_exists cargo; then
        echo "Rust is already installed"
        rustup update
        return
    fi
    
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source rust environment
    source "$HOME/.cargo/env"
    
    if command_exists rustc; then
        echo "Rust installed successfully"
        rustup update
    else
        echo "ERROR: Failed to install Rust"
        exit 1
    fi
}

# Function to install Node.js
install_nodejs() {
    if command_exists node && command_exists npm; then
        NODE_VERSION=$(node --version)
        echo "Node.js is already installed: $NODE_VERSION"
        return
    fi
    
    echo "Installing Node.js..."
    
    if command_exists nvm; then
        echo "Using nvm to install Node.js..."
        nvm install --lts
        nvm use --lts
    else
        echo "Installing nvm..."
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
        export NVM_DIR="$HOME/.nvm"
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
        [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
        
        nvm install --lts
        nvm use --lts
    fi
    
    if command_exists node; then
        echo "Node.js installed successfully: $(node --version)"
    else
        echo "ERROR: Failed to install Node.js"
        exit 1
    fi
}

# Function to install Tauri CLI
install_tauri_cli() {
    if command_exists tauri; then
        echo "Tauri CLI is already installed"
        return
    fi
    
    echo "Installing Tauri CLI..."
    cargo install tauri-cli
    
    if command_exists tauri; then
        echo "Tauri CLI installed successfully"
    else
        echo "ERROR: Failed to install Tauri CLI"
        exit 1
    fi
}

# Function to install platform-specific dependencies
install_platform_deps() {
    OS=$(detect_os)
    
    case "$OS" in
        "macos")
            echo "Installing macOS dependencies..."
            if ! command_exists xcode-select; then
                echo "ERROR: Xcode Command Line Tools not found. Please run: xcode-select --install"
                exit 1
            fi
            echo "macOS dependencies ready"
            ;;
        "linux")
            echo "Installing Linux dependencies..."
            if command_exists apt; then
                sudo apt update
                sudo apt install -y libwebkit2gtk-4.0-dev \
                    build-essential \
                    curl \
                    wget \
                    libssl-dev \
                    libgtk-3-dev \
                    libayatana-appindicator3-dev \
                    librsvg2-dev
                echo "Linux dependencies installed"
            elif command_exists yum; then
                sudo yum install -y webkitgtk4-devel \
                    gcc-c++ \
                    curl \
                    wget \
                    openssl-devel \
                    gtk3-devel \
                    librsvg2-devel
                echo "Linux dependencies installed"
            else
                echo "WARNING: Unsupported package manager. Please install dependencies manually."
            fi
            ;;
        "windows")
            echo "WARNING: Windows dependencies should be installed manually:"
            echo "- Microsoft Visual Studio C++ Build Tools"
            echo "- WebView2 Runtime"
            ;;
    esac
}

# Function to create directories
create_directories() {
    echo "Creating project directories..."
    
    mkdir -p binaries/models
    mkdir -p src-tauri/target
    
    echo "Directories created"
}

# Function to download models
download_models() {
    echo "Setting up AI models..."
    
    cd binaries/models
    
    # Download Whisper model
    if [[ ! -f "whisper-tiny.en.gguf" ]]; then
        echo "Downloading Whisper model..."
        wget -O whisper-tiny.en.gguf https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
        echo "Whisper model downloaded"
    else
        echo "Whisper model already exists"
    fi
    
    # Download platform-specific binaries
    OS=$(detect_os)
    case "$OS" in
        "macos")
            download_macos_binaries
            ;;
        "linux")
            download_linux_binaries
            ;;
        "windows")
            download_windows_binaries
            ;;
    esac
    
    cd ../..
}

# Function to download macOS binaries
download_macos_binaries() {
    cd ../..
    
    if [[ ! -f "binaries/whisperfile" ]]; then
        echo "Downloading macOS whisperfile..."
        echo "WARNING: Please download whisperfile and llamafile for macOS manually"
        echo "Place them in the binaries/ directory"
    fi
    
    if [[ ! -f "binaries/llamafile" ]]; then
        echo "Downloading macOS llamafile..."
        echo "WARNING: Please download llamafile for macOS manually"
        echo "Place it in the binaries/ directory"
    fi
}

# Function to download Linux binaries
download_linux_binaries() {
    cd ../..
    
    if [[ ! -f "binaries/whisperfile" ]]; then
        echo "Downloading Linux whisperfile..."
        echo "WARNING: Please download whisperfile and llamafile for Linux manually"
        echo "Place them in the binaries/ directory"
    fi
    
    if [[ ! -f "binaries/llamafile" ]]; then
        echo "Downloading Linux llamafile..."
        echo "WARNING: Please download llamafile for Linux manually"
        echo "Place it in the binaries/ directory"
    fi
}

# Function to download Windows binaries
download_windows_binaries() {
    cd ../..
    
    if [[ ! -f "binaries/whisperfile.exe" ]]; then
        echo "Downloading Windows whisperfile..."
        echo "WARNING: Please download whisperfile.exe and llamafile.exe for Windows manually"
        echo "Place them in the binaries/ directory"
    fi
    
    if [[ ! -f "binaries/llamafile.exe" ]]; then
        echo "Downloading Windows llamafile..."
        echo "WARNING: Please download llamafile.exe for Windows manually"
        echo "Place it in the binaries/ directory"
    fi
}

# Function to install npm dependencies
install_npm_deps() {
    echo "Installing npm dependencies..."
    
    # Ensure we're in the project root directory
    cd "$(dirname "$0")"
    
    # Debug: Show current directory and list files
    echo "Current working directory: $(pwd)"
    echo "Files in current directory:"
    ls -la
    
    if [[ -f "package.json" ]]; then
        npm install
        echo "npm dependencies installed"
    else
        echo "ERROR: package.json not found"
        exit 1
    fi
}

# Function to verify Rust dependencies
verify_rust_deps() {
    echo "Verifying Rust dependencies..."
    
    # Store the original directory
    ORIGINAL_DIR=$(pwd)
    
    cd src-tauri
    
    if cargo check; then
        echo "Rust dependencies verified"
    else
        echo "ERROR: Rust dependency verification failed"
        cd "$ORIGINAL_DIR"
        exit 1
    fi
    
    # Return to the original directory
    cd "$ORIGINAL_DIR"
}

# Function to set file permissions
set_permissions() {
    echo "Setting file permissions..."
    
    chmod +x setup.sh
    
    # Make binary files executable on Unix systems
    if [[ "$(detect_os)" != "windows" ]]; then
        if [[ -f "binaries/whisperfile" ]]; then
            chmod +x binaries/whisperfile
        fi
        if [[ -f "binaries/llamafile" ]]; then
            chmod +x binaries/llamafile
        fi
    fi
    
    echo "Permissions set"
}

# Function to run tests
run_tests() {
    echo "Running basic tests..."
    
    # Test Rust compilation
    cd src-tauri
    if cargo check; then
        echo "Rust compilation test passed"
    else
        echo "ERROR: Rust compilation test failed"
        exit 1
    fi
    cd ..
    
    # Test Node.js
    if node --version > /dev/null 2>&1; then
        echo "Node.js test passed"
    else
        echo "ERROR: Node.js test failed"
        exit 1
    fi
    
    echo "All tests passed"
}

# Function to display next steps
show_next_steps() {
    echo
    echo "Setup completed successfully!"
    echo
    echo "Next steps:"
    echo "1. Download the required binary files (whisperfile, llamafile) for your platform"
    echo "2. Place them in the binaries/ directory"
    echo "3. Download a medical Llama model and place it in binaries/models/ as med_llama.gguf"
    echo "4. Run 'npm run dev' to start development"
    echo "5. Run 'npm run build' to build for production"
    echo
    echo "For more information, see the README.md file"
}

# Main setup function
main() {
    echo "=========================================="
    echo "  Medical Note Generator Setup Script"
    echo "=========================================="
    echo
    
    # Check if we're in the right directory
    if [[ ! -f "package.json" ]] || [[ ! -f "src-tauri/Cargo.toml" ]]; then
        echo "ERROR: Please run this script from the project root directory"
        exit 1
    fi
    
    # check_system_requirements
    # install_rust
    # install_nodejs
    # install_tauri_cli
    # install_platform_deps
    # create_directories
    # download_models
    # TODO: this isn't in the right working directory so doesn't work when it's all run together
    install_npm_deps
    verify_rust_deps
    set_permissions
    run_tests
    show_next_steps
}

# Run main function
main "$@"
