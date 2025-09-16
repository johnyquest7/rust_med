<script lang="ts">
  import { setContext } from 'svelte';
  import { authContext } from '$lib/hooks/auth-context.svelte.js';
  import type { AuthContext } from '$lib/types.js';

  // Props
  let { children } = $props<{ children: any }>();

  // Set the auth context for child components
  setContext<AuthContext>('auth', authContext);
</script>

{@render children?.()}
