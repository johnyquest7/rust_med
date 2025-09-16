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

Next, set up the frontend (sveltekit) app.

```bash
cd frontend
npm install
```

With that, you're ready to run the application:

The command `npm run dev`, when executed at the root of the repo, will run (a) the frontend with `vite build --watch` such that code changes to the frontend are streamed to the `frontend/build` directory and (b) run the tauri app in dev mode with `cargo tauri dev`.

```bash
# Be at the root of the repo to start the Tauri project
cd ..
npm run dev
```

## The Codebase

### Backend Files (Rust)

- `src-tauri/src/main.rs` - Copy main.rs code from artifact
- `src-tauri/src/commands.rs` - Copy commands.rs code from artifact
- `src-tauri/tauri.conf.json` - Copy tauri.conf.json code from artifact
- `src-tauri/capabilities/default.json` - Copy default.json code from artifact

### Frontend (Sveltekit)

The frontend is a sveltekit app and located at `frontend/`. The app is a typical, static sveltekit app that's currently configured to run in prerendered mode, although that may change in the future.

The Tauri app uses whatever is in the `frontend/build` directory, so while you could develop as a typical app with `cd frontend && npm run dev`, you can always get live updates in the native Tauri app with `npm run dev` from the root of the repo. The major differences are that functionality may be limited in the web-only version because it doesn't have the rust backend and updates will be slower to take effect in the Tauri version.

## TODOs:

- Add auth method that's better to set up for encryption
- Later: Move to sqlite with encryption