<script lang="ts">
  import '../app.css';
  import favicon from '$lib/assets/favicon.svg';
  import AuthProvider from '$lib/components/auth-provider.svelte';
  import AuthenticatedLayout from '$lib/components/authenticated-layout.svelte';
  import { Toaster } from '$lib/components/ui/sonner';

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<AuthProvider>
  <AuthenticatedLayout {children} />
</AuthProvider>

<Toaster />
