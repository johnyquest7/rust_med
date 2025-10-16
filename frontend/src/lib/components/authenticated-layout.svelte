<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index.js';
  import AppSidebar from '$lib/components/app-sidebar.svelte';
  import LoginForm from '$lib/components/custom/login-form.svelte';
  import RegisterForm from '$lib/components/custom/register-form.svelte';
  import SetupWizard from '$lib/components/custom/setup-wizard.svelte';
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import type { AuthResponse } from '$lib/types.js';

  let { children } = $props();

  // Get authentication context
  const auth = useAuth();

  // State for determining which form to show
  let authFileExists = $state(false);
  let isLoading = $state(true);
  let setupCompleted = $state(false);
  let modelsInstalled = $state(false);
  let checkingSetup = $state(true);

  // Check authentication status and setup status on mount
  onMount(async () => {
    try {
      if (!browser || typeof (window as unknown as { __TAURI__?: unknown }).__TAURI__ === 'undefined') {
        isLoading = false;
        checkingSetup = false;
        return;
      }

      // Check setup status first (legacy check for database flag)
      try {
        const setupStatus = await (
          window as unknown as { __TAURI__: { core: { invoke: (cmd: string) => Promise<boolean> } } }
        ).__TAURI__.core.invoke('check_setup_status');
        setupCompleted = setupStatus;
      } catch (error) {
        console.error('Failed to check setup status:', error);
        // If setup check fails, assume setup is not completed
        setupCompleted = false;
      }

      // Check if all required models are installed
      try {
        const allModelsInstalled = await (
          window as unknown as { __TAURI__: { core: { invoke: (cmd: string) => Promise<boolean> } } }
        ).__TAURI__.core.invoke('check_all_models_installed');
        modelsInstalled = allModelsInstalled;

        // If models are not installed, we need to show the setup wizard
        // regardless of the legacy setup flag
        if (!modelsInstalled) {
          console.log('Models are not installed. Setup wizard will be shown.');
          setupCompleted = false;
        }
      } catch (error) {
        console.error('Failed to check models installation:', error);
        // If model check fails, assume models are not installed
        modelsInstalled = false;
        setupCompleted = false;
      }

      checkingSetup = false;

      // Then check auth status
      const response: AuthResponse = await (
        window as unknown as { __TAURI__: { core: { invoke: (cmd: string) => Promise<AuthResponse> } } }
      ).__TAURI__.core.invoke('check_auth_status');
      authFileExists = response.success;

      if (authFileExists && response.user) {
        // Set user directly from the response
        localStorage.setItem('auth_user', JSON.stringify(response.user));
        // Initialize auth context with the user
        auth.initialize();
      }
    } catch (error) {
      console.error('Failed to check auth status:', error);
    } finally {
      isLoading = false;
    }
  });

  // Reactive state from auth context
  let isAuthenticated = $derived(auth.state.isAuthenticated);

  // Switch between login and registration forms
</script>

{#if isLoading || checkingSetup}
  <!-- Loading state -->
  <div class="flex min-h-screen items-center justify-center">
    <div class="flex items-center gap-2">
      <div class="h-6 w-6 animate-spin rounded-full border-2 border-blue-600 border-t-transparent"></div>
      <span class="text-lg">Loading...</span>
    </div>
  </div>
{:else if !setupCompleted}
  <!-- Setup not completed - show setup wizard -->
  <SetupWizard />
{:else if isAuthenticated}
  <!-- Authenticated state - show main app -->
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
{:else}
  <!-- Not authenticated - show auth forms -->
  <div class="flex min-h-screen items-center justify-center bg-gray-50">
    <div class="w-full max-w-md">
      {#if authFileExists}
        <!-- Show login form if auth file exists -->
        <LoginForm />
      {:else}
        <!-- Show registration form if no auth file exists -->
        <RegisterForm />
      {/if}
    </div>
  </div>
{/if}
