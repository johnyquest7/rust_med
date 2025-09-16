<script lang="ts">
  import '../app.css';
  import favicon from '$lib/assets/favicon.svg';
  import * as Sidebar from '$lib/components/ui/sidebar/index.js';
  import AppSidebar from '$lib/components/app-sidebar.svelte';
  import AuthProvider from '$lib/components/auth-provider.svelte';
  import { useSidebar } from '$lib/components/ui/sidebar/context.svelte.js';

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<AuthProvider>
  <Sidebar.Provider>
    <AppSidebar />
    <main class="flex-1">
      <!-- Mobile menu trigger -->
      <div class="flex items-center gap-2 p-4 md:hidden">
        <Sidebar.Trigger class="cursor-pointer md:hidden" />
        <h1 class="text-lg font-semibold">Medical Note Generator</h1>
      </div>
      <div class="flex flex-1 flex-col gap-4 p-4">
        {@render children?.()}
      </div>
    </main>
  </Sidebar.Provider>
</AuthProvider>
