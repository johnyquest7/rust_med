# Authentication System

This project includes a comprehensive authentication system built with Svelte 5 runes and TypeScript.

## Features

- **Svelte 5 Runes**: Uses modern Svelte 5 syntax (`$state`, `$derived`, `$effect`)
- **TypeScript**: Fully typed with comprehensive interfaces
- **Context-based**: Global state management using Svelte context
- **Persistent**: User session persists across page reloads using localStorage
- **Role-based**: Support for different user roles (doctor, nurse, admin)
- **Error Handling**: Comprehensive error handling and validation
- **Accessibility**: Built with accessibility in mind

## Architecture

### Core Files

- `src/lib/types.ts` - TypeScript type definitions
- `src/lib/hooks/auth-context.svelte.ts` - Main authentication context class
- `src/lib/hooks/use-auth.svelte.ts` - Hook for accessing auth context
- `src/lib/components/auth-provider.svelte` - Context provider component

### Demo Components

- `src/lib/components/custom/login-form.svelte` - Login form component
- `src/lib/components/custom/auth-status.svelte` - Authentication status display

## Usage

### Basic Usage

```svelte
<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';

  const auth = useAuth();

  // Access authentication state
  let isAuthenticated = $derived(auth.state.isAuthenticated);
  let user = $derived(auth.state.user);
  let isLoading = $derived(auth.state.isLoading);
  let error = $derived(auth.state.error);
</script>

{#if isAuthenticated}
  <p>Welcome, {user?.name}!</p>
  <button onclick={() => auth.logout()}>Logout</button>
{:else}
  <button onclick={() => auth.login({ username: 'test', password: 'test' })}> Login </button>
{/if}
```

### Login

```typescript
// Login with credentials
await auth.login({
  username: 'doctor123',
  password: 'password123'
});
```

### Logout

```typescript
// Logout current user
auth.logout();
```

### Check Authentication Status

```typescript
// Check if user is authenticated
const isLoggedIn = auth.isAuthenticated();

// Check user role
const isDoctor = auth.hasRole('doctor');
```

### Error Handling

```typescript
try {
  await auth.login(credentials);
} catch (error) {
  // Error is automatically stored in auth.state.error
  console.error('Login failed:', error);
}
```

## State Management

The authentication system uses Svelte 5 runes for reactive state management:

- `$state` - For mutable state (user, loading, error)
- `$derived` - For computed values (isAuthenticated, state object)
- Context API - For global state sharing

## User Interface

The system includes ready-to-use components:

1. **LoginForm** - Complete login form with validation
2. **AuthStatus** - Display current authentication status
3. **AuthProvider** - Context provider for the entire app

## Security Notes

⚠️ **Current Implementation is for Development Only**

The current implementation includes:

- Mock authentication (accepts any valid username/password)
- Client-side only validation
- No server-side verification
- No password hashing

For production, you'll need to:

- Implement server-side authentication
- Add password hashing (bcrypt, Argon2, etc.)
- Add JWT tokens or session management
- Implement proper user registration
- Add password reset functionality
- Add rate limiting and security measures

## Demo

Visit the main page to see the authentication system in action. You can:

1. Enter any username (3+ characters) and password
2. See the authentication status update
3. View user information
4. Logout and login again

The system demonstrates:

- Form validation
- Loading states
- Error handling
- Persistent sessions
- Reactive UI updates
