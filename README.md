# Medical Note Generator

## Setup

### Installation

The following instructions are written for mac and linux, but should be adaptable to windows easily.

1. Install Rust with rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   (For Windows, download the setup file from https://rust-lang.org/tools/install/)
3. Restart your terminal
4. Update rust: `rustup update`
5. Install Node.js and make sure that's set up. See https://nodejs.org/.
6. Install dependencies:

```bash
# Install root dependencies
npm install

# Install frontend dependencies
cd frontend
npm install
cd ..
```

### AI Models Setup

The application will download required AI models through a built-in setup wizard on first run. No manual download is required for normal use.

**Optional: Manual Development Setup**

If you want to set up models manually for development (to avoid the setup wizard), place them in your app data directory:

```bash
# macOS/Linux
mkdir -p ~/Library/Application\ Support/com.medical.notegenerator/binaries/models

# Download models (see CLAUDE.md for specific download commands)
# Models are NOT stored in the repository
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

## Dev Installation Notes

- cd "/Users/alexledger/Library/Application Support/com.medical.notegenerator/"
