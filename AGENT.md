# Frontend Development Guide - Medical Note Generator

## Project Overview

A medical note generator that allows healthcare professionals to:
- Record patient visits
- Generate transcriptions using Whisper
- Create SOAP medical notes using MedLlama-2-7B
- Navigate and manage medical notes

## Architecture

The system consists of three main components:

### 1. Tauri Desktop App (Rust Backend)
- **Location**: `src-tauri/`
- **Purpose**: Native desktop application wrapper
- **Features**: File system access, shell commands, audio processing
- **Dependencies**: Tauri 2.0, serde, chrono

### 2. SvelteKit Frontend
- **Location**: `frontend/`
- **Purpose**: Web-based user interface
- **Tech Stack**: SvelteKit 2.x, Svelte 5, TypeScript, Tailwind CSS v4
- **Components**: shadcn-svelte UI library

### 3. AI Models & Binaries
- **Location**: `binaries/`
- **Models**: 
  - Whisper (speech-to-text): `whisperfile`
  - MedLlama-2-7B (medical note generation): `med_llama.gguf`
- **Runtime**: llamafile for model execution

### Development Workflow
- **Web Development**: `cd frontend && npm run dev` (standalone)
- **Desktop Development**: `npm run dev` (from root - builds frontend + runs Tauri)
- **Build Process**: Frontend builds to `frontend/build/` → Tauri serves from there

## Frontend Technology Stack

- **Framework**: SvelteKit 2.x with Svelte 5
- **Language**: TypeScript
- **Styling**: Tailwind CSS v4
- **UI Components**: shadcn-svelte
- **Build Tool**: Vite
- **Icons**: Lucide Svelte

## Frontend Structure (`frontend/`)

```
src/
├── routes/           # SvelteKit pages and layouts
│   ├── +page.svelte  # Page components
│   ├── +layout.svelte # Layout components
│   └── api/          # API routes
├── lib/              # Shared application code
│   ├── components/   # Reusable components
│   │   ├── ui/       # shadcn-svelte UI components
│   │   └── custom/   # Custom business components
│   ├── hooks/        # Svelte stores and reactive utilities
│   ├── utils.ts      # Utility functions
│   └── types.ts      # TypeScript definitions
└── app.html          # HTML template
```

## Development Standards

### Svelte 5 Runes (CRITICAL)
**Use Svelte 5 runes syntax - NEVER use reactive statements (`$:`) with rune-based state!**

```svelte
<script>
  // ✅ CORRECT: Using $derived with rune-based state
  let userDisplayName = $derived(auth?.state.user?.name || 'Not logged in');
  let isAuthenticated = $derived(auth?.state.isAuthenticated);
</script>

<script>
  // ❌ WRONG: Reactive statements don't work with runes
  $: userDisplayName = auth?.state.user?.name || 'Not logged in';
</script>
```

**Rune Usage:**
- `$state` - Mutable component state
- `$derived` - Computed values (including auth context)
- `$effect` - Side effects (API calls, DOM manipulation)

### TypeScript
- Use strict TypeScript configuration
- Define proper types for all props and functions
- Prefer interfaces over types for object shapes

### Components & Styling
- Create small, focused components with TypeScript interfaces
- Use Tailwind v4 utilities (prefer over custom CSS)
- Use `npx shadcn-svelte@latest add [component]` for UI components
- Don't modify auto-generated shadcn components directly

### Accessibility
- Include ARIA labels and semantic HTML
- Ensure keyboard navigation works
- Use proper heading hierarchy (h1 → h2 → h3)
- Test with screen readers and keyboard-only navigation

### Authentication Context
**Always use `$derived` for reactive auth values:**
```svelte
<script>
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  const auth = useAuth();
  
  // ✅ CORRECT
  let user = $derived(auth.state.user);
  let isAuthenticated = $derived(auth.state.isAuthenticated);
</script>
```

## Development Commands

```bash
# Full desktop app development (from root)
npm run dev

# Frontend development
cd frontend;
cd npm run dev

# Type checking
npm run check

# Linting & formatting
npm run lint
npm run lint:fix
npm run format
npm run format:check
```

## Key Patterns

### Component Props
```typescript
interface Props {
  variant?: 'default' | 'destructive' | 'outline';
  disabled?: boolean;
}
```

### File Naming
- Components: PascalCase (`UserProfile.svelte`)
- Utilities: camelCase (`formatDate.ts`)
- Pages: SvelteKit conventions (`+page.svelte`)
