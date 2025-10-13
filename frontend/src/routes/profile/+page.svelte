<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Select from '$lib/components/ui/select';
  import * as Tabs from '$lib/components/ui/tabs';
  import { tauriService } from '$lib/tauriService';
  import type { ModelInfo, ModelPreferences, WhisperModelSize, DownloadedModel } from '$lib/types';
  import User from '@lucide/svelte/icons/user';
  import Shield from '@lucide/svelte/icons/shield';
  import LogOut from '@lucide/svelte/icons/log-out';
  import CheckCircle from '@lucide/svelte/icons/check-circle';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import Settings from '@lucide/svelte/icons/settings';
  import Download from '@lucide/svelte/icons/download';
  import Trash2 from '@lucide/svelte/icons/trash-2';
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

  // Model preferences state
  let preferences: ModelPreferences | null = $state(null);
  let loadingPreferences = $state(true);
  let preferencesError: string | null = $state(null);

  // Downloaded models state
  let downloadedModels: DownloadedModel[] = $state([]);
  let loadingDownloadedModels = $state(false);

  // UI state
  let selectedWhisperSize: WhisperModelSize = $state('tiny');
  let medLlamaUrl = $state('');
  let savingPreferences = $state(false);
  let downloadingModel = $state(false);
  let downloadProgress = $state('');
  let successMessage = $state('');

  // Whisper model options with sizes
  const whisperModelOptions = [
    { value: 'tiny', label: 'Tiny (141 MB) - Fastest', size: 141 },
    { value: 'base', label: 'Base (142 MB) - Fast', size: 142 },
    { value: 'small', label: 'Small (466 MB) - Balanced', size: 466 },
    { value: 'medium', label: 'Medium (1.5 GB) - Accurate', size: 1500 },
    { value: 'large', label: 'Large (3.1 GB) - Most Accurate', size: 3100 }
  ];

  // Load models information on mount
  onMount(async () => {
    await Promise.all([
      loadModelsInfo(),
      loadPreferences(),
      loadDownloadedModels()
    ]);
  });

  async function loadModelsInfo() {
    try {
      loadingModels = true;
      modelsInfo = await tauriService.getModelsInfo();
    } catch (err) {
      console.error('Failed to load models info:', err);
      modelsError = err instanceof Error ? err.message : 'Failed to load models information';
    } finally {
      loadingModels = false;
    }
  }

  async function loadPreferences() {
    try {
      loadingPreferences = true;
      preferences = await tauriService.getModelPreferences();
      selectedWhisperSize = preferences.whisper_model_size;
      medLlamaUrl = preferences.med_llama_url;
    } catch (err) {
      console.error('Failed to load preferences:', err);
      preferencesError = err instanceof Error ? err.message : 'Failed to load preferences';
    } finally {
      loadingPreferences = false;
    }
  }

  async function loadDownloadedModels() {
    try {
      loadingDownloadedModels = true;
      downloadedModels = await tauriService.listDownloadedModels();
    } catch (err) {
      console.error('Failed to load downloaded models:', err);
    } finally {
      loadingDownloadedModels = false;
    }
  }

  async function handleSelectWhisperModel(size: WhisperModelSize) {
    if (!preferences) return;

    try {
      const whisperUrl = getWhisperModelUrl(size);
      const whisperFilename = `whisper-${size}.en.gguf`;

      // Update preferences immediately
      const newPreferences: ModelPreferences = {
        whisper_model_size: size,
        whisper_model_url: whisperUrl,
        whisper_model_filename: whisperFilename,
        med_llama_url: preferences.med_llama_url,
        med_llama_filename: preferences.med_llama_filename,
        updated_at: new Date().toISOString()
      };

      await tauriService.saveModelPreferences(newPreferences);
      preferences = newPreferences;
      selectedWhisperSize = size;
      successMessage = `Now using Whisper ${size} model`;

      // Clear success message after 2 seconds
      setTimeout(() => {
        successMessage = '';
      }, 2000);
    } catch (err) {
      console.error('Failed to update whisper preference:', err);
      preferencesError = err instanceof Error ? err.message : 'Failed to update preference';
    }
  }

  async function handleSelectMedLlamaUrl(url: string) {
    if (!preferences) return;

    try {
      // Update preferences immediately
      const newPreferences: ModelPreferences = {
        whisper_model_size: preferences.whisper_model_size,
        whisper_model_url: preferences.whisper_model_url,
        whisper_model_filename: preferences.whisper_model_filename,
        med_llama_url: url,
        med_llama_filename: preferences.med_llama_filename,
        updated_at: new Date().toISOString()
      };

      await tauriService.saveModelPreferences(newPreferences);
      preferences = newPreferences;
      medLlamaUrl = url;
      successMessage = 'MedLlama model URL updated';

      // Clear success message after 2 seconds
      setTimeout(() => {
        successMessage = '';
      }, 2000);
    } catch (err) {
      console.error('Failed to update MedLlama URL:', err);
      preferencesError = err instanceof Error ? err.message : 'Failed to update preference';
    }
  }

  async function handleDownloadWhisperModel() {
    try {
      downloadingModel = true;
      downloadProgress = 'Downloading Whisper model...';

      const whisperUrl = getWhisperModelUrl(selectedWhisperSize);
      const whisperFilename = `whisper-${selectedWhisperSize}.en.gguf`;

      await tauriService.downloadCustomModel(whisperUrl, whisperFilename);

      downloadProgress = 'Download complete!';
      successMessage = `Whisper ${selectedWhisperSize} model downloaded successfully!`;

      // Reload models and downloaded models list
      await Promise.all([loadModelsInfo(), loadDownloadedModels()]);

      // Clear messages after 3 seconds
      setTimeout(() => {
        downloadProgress = '';
        successMessage = '';
      }, 3000);
    } catch (err) {
      console.error('Failed to download model:', err);
      downloadProgress = '';
      preferencesError = err instanceof Error ? err.message : 'Failed to download model';
    } finally {
      downloadingModel = false;
    }
  }

  async function handleDownloadMedLlamaModel() {
    try {
      downloadingModel = true;
      downloadProgress = 'Downloading MedLlama model...';

      await tauriService.downloadCustomModel(medLlamaUrl, 'med_llama.gguf');

      downloadProgress = 'Download complete!';
      successMessage = 'MedLlama model downloaded successfully!';

      // Reload models and downloaded models list
      await Promise.all([loadModelsInfo(), loadDownloadedModels()]);

      // Clear messages after 3 seconds
      setTimeout(() => {
        downloadProgress = '';
        successMessage = '';
      }, 3000);
    } catch (err) {
      console.error('Failed to download model:', err);
      downloadProgress = '';
      preferencesError = err instanceof Error ? err.message : 'Failed to download model';
    } finally {
      downloadingModel = false;
    }
  }

  async function handleDeleteModel(filename: string) {
    // TODO: this confirmation wasn't working
    // if (!confirm(`Are you sure you want to delete ${filename}? This will free up storage space.`)) {
    //   return;
    // }

    try {
      await tauriService.deleteModelFile(filename);
      successMessage = `${filename} deleted successfully!`;

      // Reload models and downloaded models list
      await Promise.all([loadModelsInfo(), loadDownloadedModels()]);

      // Clear message after 3 seconds
      setTimeout(() => {
        successMessage = '';
      }, 3000);
    } catch (err) {
      console.error('Failed to delete model:', err);
      preferencesError = err instanceof Error ? err.message : 'Failed to delete model';
    }
  }

  function getWhisperModelUrl(size: WhisperModelSize): string {
    const urls = {
      tiny: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin',
      base: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin',
      small: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin',
      medium: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin',
      large: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin'
    };
    return urls[size];
  }

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

  function formatBytes(bytes: number): string {
    const mb = bytes / (1024 * 1024);
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
    return `${mb.toFixed(0)} MB`;
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

      <!-- Model Management -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center space-x-2">
            <Settings class="h-5 w-5" />
            <span>Model Management</span>
          </CardTitle>
          <CardDescription>Download, select, and manage AI models for transcription and note generation</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          {#if loadingPreferences}
            <div class="flex items-center justify-center py-8">
              <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
              <span class="ml-2 text-sm text-muted-foreground">Loading preferences...</span>
            </div>
          {:else if preferencesError}
            <Alert variant="destructive">
              <AlertDescription>{preferencesError}</AlertDescription>
            </Alert>
          {:else if preferences}
            <!-- Success Message -->
            {#if successMessage}
              <Alert>
                <CheckCircle class="h-4 w-4 text-green-600" />
                <AlertDescription>{successMessage}</AlertDescription>
              </Alert>
            {/if}

            <!-- Download Progress -->
            {#if downloadProgress}
              <Alert>
                <Loader2 class="h-4 w-4 animate-spin" />
                <AlertDescription>{downloadProgress}</AlertDescription>
              </Alert>
            {/if}

            <!-- Tabbed Interface -->
            <Tabs.Root value="active" class="w-full">
              <Tabs.List class="grid w-full grid-cols-3">
                <Tabs.Trigger value="active">Active Models</Tabs.Trigger>
                <Tabs.Trigger value="download">Download</Tabs.Trigger>
                <Tabs.Trigger value="manage">Manage Files</Tabs.Trigger>
              </Tabs.List>

              <!-- Tab 1: Select Active Models -->
              <Tabs.Content value="active" class="space-y-6 pt-4">
                <!-- Active Whisper Model Selection -->
                <div class="space-y-3">
                  <Label class="text-base font-semibold">Active Whisper Model</Label>
                  <p class="text-sm text-muted-foreground">
                    Select which downloaded Whisper model to use for transcription. Changes are saved automatically.
                  </p>

                  <div class="space-y-3">
                    {#each whisperModelOptions as option}
                      {@const isDownloaded = downloadedModels.some(m => m.filename === `whisper-${option.value}.en.gguf`)}
                      {@const isActive = preferences.whisper_model_size === option.value}
                      <button
                        onclick={() => isDownloaded && handleSelectWhisperModel(option.value)}
                        disabled={!isDownloaded}
                        class="w-full flex items-center justify-between rounded-lg border p-4 text-left transition-colors {isActive ? 'border-primary bg-primary/5' : 'hover:bg-muted/50'} {!isDownloaded ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}"
                      >
                        <div class="flex-1">
                          <p class="text-sm font-medium">{option.label}</p>
                          <p class="text-xs text-muted-foreground">
                            {isDownloaded ? 'Downloaded' : 'Not downloaded'}
                          </p>
                        </div>
                        {#if isActive}
                          <CheckCircle class="h-5 w-5 text-primary" />
                        {/if}
                      </button>
                    {/each}
                  </div>
                </div>

                <Separator />

                <!-- Active MedLlama URL -->
                <div class="space-y-3">
                  <Label class="text-base font-semibold">Active MedLlama Model URL</Label>
                  <p class="text-sm text-muted-foreground">
                    The URL of the MedLlama model currently being used for note generation.
                  </p>

                  <div class="rounded-lg bg-muted/50 p-4">
                    <p class="text-sm font-medium mb-2">Current URL:</p>
                    <p class="break-all text-xs font-mono text-muted-foreground">{preferences.med_llama_url}</p>
                  </div>
                </div>
              </Tabs.Content>

              <!-- Tab 2: Download Models -->
              <Tabs.Content value="download" class="space-y-6 pt-4">
                <!-- Whisper Model Download -->
                <div class="space-y-3">
                  <Label class="text-base font-semibold">Download Whisper Model</Label>
                  <p class="text-sm text-muted-foreground">
                    Choose and download a Whisper model for transcription. Larger models are more accurate but slower and use more storage.
                  </p>

                  <div class="grid gap-4 md:grid-cols-2">
                    <div class="space-y-2">
                      <Label for="whisper-download-size">Model Size</Label>
                      <Select.Root
                        type="single"
                        bind:value={selectedWhisperSize}
                      >
                        <Select.Trigger id="whisper-download-size" class="w-full">
                          {whisperModelOptions.find(opt => opt.value === selectedWhisperSize)?.label || 'Select model size'}
                        </Select.Trigger>
                        <Select.Content>
                          {#each whisperModelOptions as option}
                            <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
                          {/each}
                        </Select.Content>
                      </Select.Root>
                    </div>

                    <div class="flex items-end">
                      <Button
                        onclick={handleDownloadWhisperModel}
                        disabled={downloadingModel}
                        variant="outline"
                        class="w-full"
                      >
                        {#if downloadingModel}
                          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                          Downloading...
                        {:else}
                          <Download class="mr-2 h-4 w-4" />
                          Download
                        {/if}
                      </Button>
                    </div>
                  </div>
                </div>

                <Separator />

                <!-- MedLlama Model Download -->
                <div class="space-y-3">
                  <Label class="text-base font-semibold">Download MedLlama Model</Label>
                  <p class="text-sm text-muted-foreground">
                    Enter a URL to download a MedLlama model for note generation. This should be a direct download link to a .gguf file.
                  </p>

                  <div class="space-y-2">
                    <Label for="medllama-download-url">Model URL</Label>
                    <Input
                      id="medllama-download-url"
                      type="url"
                      bind:value={medLlamaUrl}
                      placeholder="https://huggingface.co/..."
                      class="font-mono text-sm"
                    />
                  </div>

                  <Button
                    onclick={handleDownloadMedLlamaModel}
                    disabled={downloadingModel || !medLlamaUrl}
                    variant="outline"
                    class="w-full"
                  >
                    {#if downloadingModel}
                      <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                      Downloading...
                    {:else}
                      <Download class="mr-2 h-4 w-4" />
                      Download
                    {/if}
                  </Button>
                </div>
              </Tabs.Content>

              <!-- Tab 3: Manage Downloaded Files -->
              <Tabs.Content value="manage" class="space-y-4 pt-4">
                <div class="space-y-3">
                  <Label class="text-base font-semibold">Downloaded Model Files</Label>
                  <p class="text-sm text-muted-foreground">
                    Manage your downloaded model files. Delete unused models to free up storage space.
                  </p>

                  {#if loadingDownloadedModels}
                    <div class="flex items-center justify-center py-8">
                      <Loader2 class="h-4 w-4 animate-spin text-muted-foreground" />
                      <span class="ml-2 text-sm text-muted-foreground">Loading models...</span>
                    </div>
                  {:else if downloadedModels.length === 0}
                    <div class="rounded-lg border border-dashed p-8 text-center">
                      <p class="text-sm text-muted-foreground">No model files found in the models directory.</p>
                      <p class="text-xs text-muted-foreground mt-2">Download models from the Download tab to get started.</p>
                    </div>
                  {:else}
                    <div class="space-y-2">
                      {#each downloadedModels as model}
                        <div class="flex items-center justify-between rounded-lg border p-3">
                          <div class="flex-1">
                            <p class="text-sm font-medium font-mono">{model.filename}</p>
                            <p class="text-xs text-muted-foreground">{formatBytes(model.size_bytes)}</p>
                          </div>
                          <Button
                            variant="destructive"
                            size="sm"
                            onclick={() => handleDeleteModel(model.filename)}
                          >
                            <Trash2 class="h-4 w-4" />
                          </Button>
                        </div>
                      {/each}
                    </div>

                    <!-- Total Downloaded Size -->
                    <div class="rounded-lg bg-muted/50 p-3 text-sm font-medium">
                      Total: {formatBytes(downloadedModels.reduce((sum, m) => sum + m.size_bytes, 0))}
                    </div>
                  {/if}
                </div>
              </Tabs.Content>
            </Tabs.Root>
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
