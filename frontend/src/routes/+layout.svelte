<script lang="ts">
  import '../app.css';
  import favicon from '$lib/assets/favicon.svg';
  import * as Sidebar from '$lib/components/ui/sidebar/index.js';
  import AppSidebar from '$lib/components/app-sidebar.svelte';
  import AuthProvider from '$lib/components/auth-provider.svelte';
  import LoginForm from '$lib/components/custom/login-form.svelte';
  import RegisterForm from '$lib/components/custom/register-form.svelte';
  import { useSidebar } from '$lib/components/ui/sidebar/context.svelte.js';
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import type { AuthResponse } from '$lib/types.js';

  let { children } = $props();
  
  // Get authentication context
  const auth = useAuth();
  
  // State for determining which form to show
  let showRegistration = $state(false);
  let authFileExists = $state(false);
  let isLoading = $state(true);

  // Check authentication status on mount
  onMount(async () => {
    try {
      if (!browser || typeof (window as any).__TAURI__ === 'undefined') {
        isLoading = false;
        return;
      }
      const response: AuthResponse = await (window as any).__TAURI__.core.invoke('check_auth_status');
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
  let authError = $derived(auth.state.error);

  // Switch between login and registration forms
  function switchToRegistration() {
    showRegistration = true;
    auth.clearError();
  }

  function switchToLogin() {
    showRegistration = false;
    auth.clearError();
  }
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<AuthProvider>
  {#if isLoading}
    <!-- Loading state -->
    <div class="flex min-h-screen items-center justify-center">
      <div class="flex items-center gap-2">
        <div class="h-6 w-6 animate-spin rounded-full border-2 border-blue-600 border-t-transparent"></div>
        <span class="text-lg">Loading...</span>
      </div>
    </div>
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
          <div class="mt-4 text-center">
            <button
              onclick={switchToRegistration}
              class="text-sm text-blue-600 hover:text-blue-800 underline"
            >
              Create a new account instead
            </button>
          </div>
        {:else}
          <!-- Show registration form if no auth file exists -->
          <RegisterForm />
          <div class="mt-4 text-center">
            <button
              onclick={switchToLogin}
              class="text-sm text-blue-600 hover:text-blue-800 underline"
            >
              Already have an account? Login
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</AuthProvider>
