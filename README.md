# Medical Note Generator

## Setup

### Installation

The following instructions are written for mac and linux, but should be adaptable to windows easily.

1. Install Rust with rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Restart your terminal
3. Update rust: `rustup update`
4. Install the tauri-cli: `cargo install tauri-cli`
5. Install Node.js and make sure that's set up. See https://nodejs.org/.
6. Install node dependencies: `npm install`
7. Download the whisper executable, llama executable, and the med_llama llm weights

```bash
mkdir -p binaries
mkdir -p binaries/models

wget https://huggingface.co/Mozilla/whisperfile/resolve/main/whisper-tiny.en.llamafile
mv whisper-tiny.en.llamafile binaries/whisperfile

curl -L -o llamafile "https://github.com/Mozilla-Ocho/llamafile/releases/download/0.9.3/llamafile-0.9.3"
mv llamafile binaries

curl -L -o med_llama.gguf https://huggingface.co/garcianacho/MedLlama-2-7B-GGUF/resolve/main/MedLlama-2-7B.q4_K_S.gguf?download=true
mv med_llama.gguf binaries/models
```

With that, you're ready to run the application:

```bash
npm run dev
```

## The Codebase

### Backend Files (Rust)

- `src-tauri/src/main.rs` - Copy main.rs code from artifact
- `src-tauri/src/commands.rs` - Copy commands.rs code from artifact
- `src-tauri/tauri.conf.json` - Copy tauri.conf.json code from artifact
- `src-tauri/capabilities/default.json` - Copy default.json code from artifact

### Frontend Files (Web)

- `src/index.html` - Copy index.html code from artifact
- `src/styles.css` - Copy styles.css code from artifact
- `src/main.js` - Copy main.js code from artifact
