<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';

  // Get authentication context
  const auth = useAuth();

  // Reactive state from auth context
  let state = $derived(auth.state);
  let user = $derived(state.user);
  let isAuthenticated = $derived(state.isAuthenticated);
  let isLoading = $derived(state.isLoading);
  let error = $derived(state.error);

  /**
   * Handle logout
   */
  function handleLogout() {
    auth.logout();
  }

  /**
   * Clear any authentication errors
   */
  function handleClearError() {
    auth.clearError();
  }
</script>

<Card class="w-full max-w-md">
  <CardHeader>
    <CardTitle>Authentication Status</CardTitle>
    <CardDescription>Current authentication state and user information</CardDescription>
  </CardHeader>
  <CardContent class="space-y-4">
    {#if isLoading}
      <div class="flex items-center gap-2">
        <div class="h-4 w-4 animate-spin rounded-full border-2 border-gray-300 border-t-blue-600"></div>
        <span class="text-sm text-gray-600">Loading...</span>
      </div>
    {:else if isAuthenticated && user}
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium">Status:</span>
          <Badge variant="default">Authenticated</Badge>
        </div>
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-600">Username:</span>
            <span class="text-sm font-medium">{user.username}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-600">Name:</span>
            <span class="text-sm font-medium">{user.name}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-600">Email:</span>
            <span class="text-sm font-medium">{user.email}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-600">Specialty:</span>
            <Badge variant="secondary">{user.specialty}</Badge>
          </div>
        </div>
        <Button variant="outline" class="w-full" onclick={handleLogout}>Logout</Button>
      </div>
    {:else}
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium">Status:</span>
          <Badge variant="secondary">Not Authenticated</Badge>
        </div>
        <p class="text-sm text-gray-600">No user is currently logged in.</p>
      </div>
    {/if}

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
  </CardContent>
</Card>
