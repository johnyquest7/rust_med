# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Medical Note Generator is a privacy-focused Tauri desktop application that records doctor-patient conversations, transcribes them using Whisper, and generates medical notes (SOAP or detailed) using a local LLM. All processing happens locally with encrypted storage.

## Development Commands

```bash
# Install dependencies (run from project root)
npm install
cd frontend && npm install

# Run in development mode (runs both frontend watch and Tauri dev)
npm run dev

# Build for production
npm run build
# or
cargo tauri build

# Frontend-only development (from frontend/ directory)
cd frontend
npm run dev          # Run dev server
npm run build        # Build frontend
npm run lint         # Lint code
npm run format       # Format code with Prettier
npm run check        # Type check with svelte-check
```

## Initial Setup

The application requires downloading AI binaries before first run:

```bash
mkdir -p binaries/models

# Download Whisper model (speech-to-text)
wget https://huggingface.co/Mozilla/whisperfile/resolve/main/whisper-tiny.en.llamafile
mv whisper-tiny.en.llamafile binaries/whisperfile
chmod +x binaries/whisperfile

# Download Llamafile (LLM runtime)
curl -L -o llamafile "https://github.com/Mozilla-Ocho/llamafile/releases/download/0.9.3/llamafile-0.9.3"
mv llamafile binaries/
chmod +x binaries/llamafile

# Download Medical LLaMA model (note generation)
curl -L -o med_llama.gguf https://huggingface.co/garcianacho/MedLlama-2-7B-GGUF/resolve/main/MedLlama-2-7B.q4_K_S.gguf?download=true
mv med_llama.gguf binaries/models/
```

## Architecture

### Technology Stack

- **Frontend**: SvelteKit 2 with static adapter and Svelte 5, Tailwind CSS 4, TypeScript
- **Backend**: Rust + Tauri 2.0
- **Authentication**: Argon2id for password hashing, AES-256-GCM for encryption
- **AI Models**: Local Whisper (transcription), MedLlama (note generation)

### Frontend Architecture (SvelteKit)

The frontend is located in `frontend/` and uses SvelteKit in static/prerendered mode:

- **Entry Point**: `frontend/src/routes/+page.svelte`
- **Component Library**: Custom components in `frontend/src/lib/components/`
  - `auth-provider.svelte`: Global authentication state management
  - `authenticated-layout.svelte`: Protected layout wrapper
  - `custom/`: Business logic components (login, register, tables)
  - `ui/`: Reusable UI components built on bits-ui/shadcn-svelte
- **Build Output**: `frontend/build/` (consumed by Tauri)
- **Development Flow**: `npm run dev` at root runs `vite build --watch` alongside `cargo tauri dev` for live updates

#### Using shadcn-svelte Components

This project uses [shadcn-svelte](https://shadcn-svelte.com/) for UI components, which are based on bits-ui primitives. Components are stored locally in `frontend/src/lib/components/ui/`.

**Adding New Components:**

```bash
# From frontend/ directory
npx shadcn-svelte@latest add <component-name>

# Examples:
npx shadcn-svelte@latest add button
npx shadcn-svelte@latest add dialog
npx shadcn-svelte@latest add table
```

**Using Components in Your Code:**

```svelte
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
</script>

<Card>
  <CardHeader>
    <CardTitle>Example Card</CardTitle>
  </CardHeader>
  <CardContent>
    <Button>Click me</Button>
  </CardContent>
</Card>
```

**Icon Usage (Optimized for Build Performance):**

Use direct imports from `@lucide/svelte` to minimize bundle size and build time:

```svelte
<script lang="ts">
  // ✅ DO: Direct imports (fast builds, small bundles)
  import User from '@lucide/svelte/icons/user';
  import Shield from '@lucide/svelte/icons/shield';
  import LogOut from '@lucide/svelte/icons/log-out';

  // ❌ DON'T: Barrel imports (slow builds, large bundles)
  // import { User, Shield, LogOut } from '@lucide/svelte';
</script>

<User class="h-4 w-4" />
```

**Styling Components:**

- Components use Tailwind CSS 4 for styling
- Custom styles can be passed via the `class` prop
- Use `tailwind-merge` (imported as `cn` utility) for combining classes:

```svelte
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { cn } from '$lib/utils';
</script>

<Button class={cn("bg-blue-500", someCondition && "bg-red-500")}>
  Styled Button
</Button>
```

### Backend Architecture (Rust)

Core files in `src-tauri/src/`:

1. **main.rs** - Primary Tauri commands and application setup
   - Audio recording via Web Audio API (browser-based)
   - Transcription: Executes whisperfile binary with `--no-prints` flag
   - Note generation: Streams llamafile output with real-time events
   - Patient note CRUD operations (encrypted storage)
   - Authentication commands

2. **auth.rs** - Zero-knowledge encryption system
   - Password-based key derivation (Argon2id)
   - Data Encryption Key (DEK) wrapped with user password
   - AES-256-GCM encryption for patient notes
   - No passwords stored, only password hashes for DEK unwrapping

3. **db.rs** - SQLite database operations
   - Database initialization and schema management
   - CRUD operations for authentication data
   - CRUD operations for encrypted patient notes
   - All data stored in SQLite for reliability and performance

4. **commands.rs** - Alternative recording implementation (appears unused)
   - References `tauri-plugin-mic-recorder` (not in current Cargo.toml)
   - Likely legacy code from earlier architecture iteration

### AI Processing Pipeline

1. **Audio Recording**:
   - Web Audio API captures microphone input
   - Converts to WAV format in browser
   - Saves to Tauri app local data directory

2. **Transcription** (whisperfile):
   - Binary paths resolved for dev/prod environments
   - Model search order: `whisper-tiny.en.gguf` → fallback models
   - Output parsing: `[00:00:00.000 --> 00:00:05.000] text` format
   - Validation: Checks WAV headers, file size, audio content

3. **Note Generation** (llamafile):
   - Two note types: SOAP (structured) or Full (detailed 13-section)
   - Uses LLaMA chat template with medical prompt engineering
   - Streaming output via events (`note-generation-stream`, `note-generation-complete`)
   - Output cleaning removes chat artifacts and ensures formatting

4. **Storage**:
   - Patient notes encrypted with DEK before saving
   - Stored in SQLite database (`medical_notes.db`)
   - All data persists in `{app_local_data_dir}/`

### Key Implementation Details

- **Binary Path Resolution**: Checks multiple locations for dev (project root) and prod (resource dir)
- **Absolute Paths Required**: Llamafile model loading needs canonicalized paths on Windows
- **Streaming Output**: LLM generation emits real-time events for UI updates
- **Error Handling**: Comprehensive validation with user-friendly error messages
- **Whisper Output Parsing**: Handles timestamped format and blank audio detection
- **LLM Output Cleaning**: Removes template artifacts, stops on repetition/conversation continuation
- **Encryption Flow**: Password → Argon2 → KEK → Unwrap DEK → AES-GCM encrypt/decrypt data

### Security Architecture

- **Zero-Knowledge Design**: Password never stored, only used to derive KEK
- **DEK Wrapping**: Random 256-bit DEK encrypted with password-derived key
- **Data Encryption**: All patient notes encrypted at rest with AES-256-GCM
- **Key Derivation**: Argon2id with 64MB memory, 3 iterations, parallelism=2

### Data Storage Locations

- **App Data**: `~/Library/Application Support/com.medical.notegenerator/` (macOS)
- **Database**: `medical_notes.db` SQLite database (contains auth and encrypted notes)
- **Audio Files**: Temporary WAV files in app data directory

### Important Patterns

- **Tauri Commands**: All backend functions exposed via `#[tauri::command]` macro
- **Event Emission**: Backend emits progress events for long-running operations
- **Frontend Auth Flow**: AuthProvider manages global authentication state with Svelte 5 runes
- **Static Generation**: Frontend prerendered for Tauri embedding (no SSR)
- **Concurrency**: `npm run dev` uses `concurrently` to run frontend watch + Tauri dev
