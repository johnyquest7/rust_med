<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import type { LoginCredentials } from '$lib/types.js';

  // Get authentication context
  const auth = useAuth();

  // Form state
  let credentials = $state<LoginCredentials>({
    username: '',
    password: ''
  });

  let errors = $state<Record<string, string>>({});

  // Reactive state from auth context
  let isLoading = $derived(auth.state.isLoading);
  let error = $derived(auth.state.error);

  /**
   * Validate form inputs
   */
  function validateForm(): boolean {
    errors = {};

    if (!credentials.username.trim()) {
      errors.username = 'Username is required';
    } else if (credentials.username.length < 3) {
      errors.username = 'Username must be at least 3 characters long';
    }

    if (!credentials.password.trim()) {
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
      await auth.login(credentials);
      // Reset form on successful login
      credentials = { username: '', password: '' };
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
    <CardDescription>Enter your username and password to access the system</CardDescription>
  </CardHeader>
  <CardContent>
    <form onsubmit={handleSubmit} class="space-y-4">
      <div class="space-y-2">
        <Label for="username">Username</Label>
        <Input
          id="username"
          type="text"
          placeholder="Enter your username"
          bind:value={credentials.username}
          disabled={isLoading}
          class={errors.username ? 'border-red-500' : ''}
        />
        {#if errors.username}
          <p class="text-sm text-red-600">{errors.username}</p>
        {/if}
      </div>

      <div class="space-y-2">
        <Label for="password">Password</Label>
        <Input
          id="password"
          type="password"
          placeholder="Enter your password"
          bind:value={credentials.password}
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

    <div class="mt-4 rounded-md bg-blue-50 p-3">
      <p class="text-sm text-blue-800">
        <strong>Demo:</strong> Enter any username (3+ characters) and password to login.
      </p>
    </div>
  </CardContent>
</Card>
