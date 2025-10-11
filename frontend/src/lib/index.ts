// place files you want to import through the `$lib` alias in this folder.

// Export authentication utilities
export { authContext } from './hooks/auth-context.svelte.js';
export { useAuth } from './hooks/use-auth.svelte.js';
export type { User, AuthState, LoginCredentials, RegisterData, AuthContext } from './types.js';
