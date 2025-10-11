<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import User from '@lucide/svelte/icons/user';
  import Shield from '@lucide/svelte/icons/shield';
  import LogOut from '@lucide/svelte/icons/log-out';
  import { goto } from '$app/navigation';

  // Get authentication context
  const auth = useAuth();

  // Reactive state from auth context
  let state = $derived(auth.state);
  let user = $derived(state.user);
  let isAuthenticated = $derived(state.isAuthenticated);
  let error = $derived(state.error);

  function handleLogout() {
    auth.logout();
    goto('/');
  }

  function handleClearError() {
    auth.clearError();
  }
</script>

<svelte:head>
  <title>Profile - Medical Note Generator</title>
  <meta name="description" content="User profile and account settings for Medical Note Generator." />
</svelte:head>

{#if !isAuthenticated}
  <div class="container mx-auto px-4 py-16">
    <div class="mx-auto max-w-md">
      <Alert>
        <AlertDescription>You need to be logged in to view your profile. Please log in first.</AlertDescription>
      </Alert>
    </div>
  </div>
{:else if user}
  <div class="container mx-auto px-4 py-8">
    <div class="mx-auto max-w-4xl space-y-8">
      <!-- Profile Information and Settings -->
      <!-- Account Information -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center space-x-2">
            <User class="h-5 w-5" />
            <span>Your Account</span>
          </CardTitle>
          <CardDescription>Your account details and information</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between py-2">
            <span class="text-sm font-medium text-gray-600">Username</span>
            <span class="text-sm">{user.username}</span>
          </div>
          <Separator />
          <div class="flex items-center justify-between py-2">
            <span class="text-sm font-medium text-gray-600">User ID</span>
            <span class="font-mono text-sm text-gray-500">{user.user_id}</span>
          </div>
          <Separator />
          <div class="flex items-center justify-between py-2">
            <span class="text-sm font-medium text-gray-600">Account Status</span>
            <Badge variant="default">Active</Badge>
          </div>
        </CardContent>
      </Card>

      <!-- Security Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center space-x-2">
            <Shield class="h-5 w-5" />
            <span>Security Settings</span>
          </CardTitle>
          <CardDescription>Manage your account security and privacy settings</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <!-- TODO: changing pasword not implemented -->
          <!-- <div class="flex items-center justify-between py-2">
            <div class="flex items-center space-x-3">
              <Key class="h-4 w-4 text-gray-500" />
              <div>
                <p class="text-sm font-medium">Password</p>
                <p class="text-xs text-gray-500">Last changed 2 months ago</p>
              </div>
            </div>
            <Button variant="outline" size="sm">Change Password</Button>
          </div>
          <Separator /> -->
          <div class="flex items-center justify-between py-2">
            <div class="flex items-center space-x-3">
              <LogOut class="h-4 w-4 text-gray-500" />
              <div>
                <p class="text-sm font-medium">Sign Out</p>
                <p class="text-xs text-gray-500">Sign out of your account</p>
              </div>
            </div>
            <Button variant="destructive" size="sm" onclick={handleLogout}>Sign Out</Button>
          </div>
        </CardContent>
      </Card>

      <!-- Error Display -->
      {#if error}
        <Alert variant="destructive">
          <AlertDescription>
            {error}
            <Button variant="ghost" size="sm" onclick={handleClearError} class="ml-2 h-6 w-6 p-0">×</Button>
          </AlertDescription>
        </Alert>
      {/if}
    </div>
  </div>
{/if}
