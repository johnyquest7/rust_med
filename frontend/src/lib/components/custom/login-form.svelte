<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { browser } from '$app/environment';
  import type { AuthResponse } from '$lib/types.js';

  // Get authentication context
  const auth = useAuth();

  // Form state
  let password = $state('');
  let username = $state('');
  let errors = $state<Record<string, string>>({});

  // Reactive state from auth context
  let isLoading = $derived(auth.state.isLoading);
  let error = $derived(auth.state.error);

  // Load username from auth file on component mount
  async function loadUsername() {
    try {
      if (!browser || typeof (window as any).__TAURI__ === 'undefined') {
        return;
      }
      const response: AuthResponse = await (window as any).__TAURI__.core.invoke('get_user_info_command');
      if (response.success && response.user) {
        username = response.user.username;
      }
    } catch (error) {
      console.error('Failed to load username:', error);
    }
  }

  // Load username when component mounts
  loadUsername();

  /**
   * Validate form inputs
   */
  function validateForm(): boolean {
    errors = {};

    if (!password.trim()) {
      errors.password = 'Password is required';
    }

    return Object.keys(errors).length === 0;
  }

  /**
   * Handle form submission
   */
  async function handleSubmit(event: Event) {
    event.preventDefault();

    if (!validateForm()) {
      return;
    }

    try {
      await auth.login(password);
      // Reset form on successful login
      password = '';
    } catch (error) {
      // Error is handled by the auth context
      console.error('Login failed:', error);
    }
  }

  /**
   * Clear authentication error
   */
  function handleClearError() {
    auth.clearError();
  }
</script>

<Card class="w-full max-w-md">
  <CardHeader>
    <CardTitle>Login</CardTitle>
    <CardDescription>Enter your password to access the system</CardDescription>
  </CardHeader>
  <CardContent>
    <form onsubmit={handleSubmit} class="space-y-4">
      {#if username}
        <div class="space-y-2">
          <Label>Username</Label>
          <div class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700">
            {username}
          </div>
        </div>
      {/if}

      <div class="space-y-2">
        <Label for="password">Password</Label>
        <Input
          id="password"
          type="password"
          placeholder="Enter your password"
          bind:value={password}
          disabled={isLoading}
          class={errors.password ? 'border-red-500' : ''}
        />
        {#if errors.password}
          <p class="text-sm text-red-600">{errors.password}</p>
        {/if}
      </div>

      {#if error}
        <div class="rounded-md bg-red-50 p-3">
          <div class="flex items-center justify-between">
            <p class="text-sm text-red-800">{error}</p>
            <Button
              variant="ghost"
              size="sm"
              onclick={handleClearError}
              class="h-6 w-6 p-0 text-red-600 hover:bg-red-100"
            >
              ×
            </Button>
          </div>
        </div>
      {/if}

      <Button type="submit" class="w-full" disabled={isLoading}>
        {#if isLoading}
          <div class="flex items-center gap-2">
            <div class="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
            <span>Logging in...</span>
          </div>
        {:else}
          Login
        {/if}
      </Button>
    </form>
  </CardContent>
</Card>
