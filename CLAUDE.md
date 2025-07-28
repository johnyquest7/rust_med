# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Medical Note Generator is a Tauri-based desktop application that records doctor-patient conversations, transcribes them using Whisper, and generates SOAP notes using a local LLM (llamafile).

## Development Commands

```bash
# Install dependencies
npm install

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

## Architecture

### Core Components

1. **Frontend (Web)**: Vanilla JavaScript modules in `src/`
   - `main.js`: Main entry point and app initialization
   - `modules/`: Modular components for UI, recording, transcription management
   - Uses Web Audio API for browser-based audio recording

2. **Backend (Rust)**: Tauri application in `src-tauri/`
   - `main.rs`: Core Tauri commands for transcription and note generation
   - `commands.rs`: Additional commands (appears to be for a different recording approach)
   - Uses `tauri-plugin-shell` to execute whisperfile and llamafile binaries

3. **AI Components**: Local binaries in `binaries/`
   - `whisperfile.exe`: Speech-to-text transcription
   - `llamafile.exe`: Medical note generation
   - Models in `binaries/models/`: `whisper-tiny.en.gguf`, `med_llama.gguf`

### Key Implementation Details

- **Audio Recording**: Web Audio API captures audio, converts to WAV format
- **File Handling**: Audio saved to Tauri app local data directory
- **Transcription**: Whisperfile executed with `--no-prints` flag to get clean output
- **Note Generation**: Llamafile uses SOAP format prompt template with specific parameters for medical context
- **Error Handling**: Comprehensive validation for audio files, binary paths, and model availability

### Important Patterns

- Binary paths are resolved dynamically for both development and production environments
- Whisperfile output parsing handles timestamped format `[00:00:00.000 --> 00:00:05.000] text`
- LLM output cleaning removes chat template artifacts and ensures proper SOAP formatting
- Absolute paths required for llamafile model loading on Windows