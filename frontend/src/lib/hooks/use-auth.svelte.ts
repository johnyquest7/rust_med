import { getContext } from 'svelte';
import type { AuthContext } from '$lib/types.js';

/**
 * Hook to access authentication context in components
 * @returns Authentication context
 * @throws Error if used outside of AuthProvider
 */
export function useAuth(): AuthContext {
  const auth = getContext<AuthContext>('auth');

  if (!auth) {
    throw new Error('useAuth must be used within an AuthProvider');
  }

  return auth;
}
