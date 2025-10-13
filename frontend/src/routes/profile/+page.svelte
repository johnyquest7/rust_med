<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { tauriService } from '$lib/tauriService';
  import type { ModelInfo } from '$lib/types';
  import User from '@lucide/svelte/icons/user';
  import Shield from '@lucide/svelte/icons/shield';
  import LogOut from '@lucide/svelte/icons/log-out';
  import Database from '@lucide/svelte/icons/database';
  import CheckCircle from '@lucide/svelte/icons/check-circle';
  import XCircle from '@lucide/svelte/icons/x-circle';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  // Get authentication context
  const auth = useAuth();

  // Reactive state from auth context
  let state = $derived(auth.state);
  let user = $derived(state.user);
  let isAuthenticated = $derived(state.isAuthenticated);
  let error = $derived(state.error);

  // Model information state
  let modelsInfo: ModelInfo[] = $state([]);
  let loadingModels = $state(true);
  let modelsError: string | null = $state(null);

  // Load models information on mount
  onMount(async () => {
    try {
      modelsInfo = await tauriService.getModelsInfo();
    } catch (err) {
      console.error('Failed to load models info:', err);
      modelsError = err instanceof Error ? err.message : 'Failed to load models information';
    } finally {
      loadingModels = false;
    }
  });

  function handleLogout() {
    auth.logout();
    goto('/');
  }

  function handleClearError() {
    auth.clearError();
  }

  function formatFileSize(sizeMb: number): string {
    if (sizeMb >= 1024) {
      return `${(sizeMb / 1024).toFixed(2)} GB`;
    }
    return `${sizeMb.toFixed(0)} MB`;
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

      <!-- AI Models Information -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center space-x-2">
            <Database class="h-5 w-5" />
            <span>AI Models</span>
          </CardTitle>
          <CardDescription>Local AI models used for transcription and note generation</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          {#if loadingModels}
            <div class="flex items-center justify-center py-8">
              <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
              <span class="ml-2 text-sm text-muted-foreground">Loading model information...</span>
            </div>
          {:else if modelsError}
            <Alert variant="destructive">
              <AlertDescription>{modelsError}</AlertDescription>
            </Alert>
          {:else if modelsInfo.length > 0}
            {#each modelsInfo as model, index}
              <div class="space-y-2">
                <div class="flex items-start justify-between">
                  <div class="flex-1 space-y-1">
                    <div class="flex items-center space-x-2">
                      {#if model.installed}
                        <CheckCircle class="h-4 w-4 text-green-600" />
                      {:else}
                        <XCircle class="h-4 w-4 text-red-600" />
                      {/if}
                      <p class="text-sm font-medium">{model.name}</p>
                    </div>
                    <div class="ml-6 space-y-0.5">
                      <p class="text-xs text-muted-foreground">
                        File: <span class="font-mono">{model.file_name}</span>
                      </p>
                      <p class="text-xs text-muted-foreground">Size: {formatFileSize(model.size_mb)}</p>
                      {#if model.installed && model.file_path}
                        <p class="text-xs text-muted-foreground">
                          Status: <span class="text-green-600">Installed</span>
                        </p>
                      {:else}
                        <p class="text-xs text-red-600">Status: Not installed</p>
                      {/if}
                    </div>
                  </div>
                  <Badge variant={model.installed ? 'default' : 'destructive'}>
                    {model.installed ? 'Active' : 'Missing'}
                  </Badge>
                </div>
                {#if index < modelsInfo.length - 1}
                  <Separator />
                {/if}
              </div>
            {/each}

            <!-- Total storage usage -->
            <Separator />
            <div class="flex items-center justify-between rounded-lg bg-muted/50 px-4 py-3">
              <span class="text-sm font-medium">Total Storage Used</span>
              <span class="text-sm font-semibold">
                {formatFileSize(
                  modelsInfo
                    .filter((m: ModelInfo) => m.installed)
                    .reduce((sum: number, m: ModelInfo) => sum + m.size_mb, 0)
                )}
              </span>
            </div>
          {:else}
            <p class="text-sm text-muted-foreground">No model information available.</p>
          {/if}
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
