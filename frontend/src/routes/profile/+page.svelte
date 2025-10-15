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
  import type {
    ModelPreferences,
    WhisperModelSize,
    DownloadedModel,
    WhisperModelMetadata,
    MedLlamaModelMetadata,
    ModelInfo
  } from '$lib/types';
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
  let authState = $derived(auth.state);
  let user = $derived(authState.user);
  let isAuthenticated = $derived(authState.isAuthenticated);
  let error = $derived(authState.error);

  // Model information state
  let modelsInfo: ModelInfo[] = $state([]);
  let loadingModels = $state(false);
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
  let downloadingModel = $state(false);
  let downloadProgress = $state('');
  let successMessage = $state('');

  // Model metadata from backend (SINGLE SOURCE OF TRUTH)
  let whisperModelOptions = $state<WhisperModelMetadata[]>([]);
  let medllamaMetadata = $state<MedLlamaModelMetadata | null>(null);

  // Load models information on mount
  onMount(async () => {
    await Promise.all([loadModelMetadata(), loadModelsInfo(), loadPreferences(), loadDownloadedModels()]);
  });

  async function loadModelMetadata() {
    try {
      // Load all metadata from backend (SINGLE SOURCE OF TRUTH)
      const [whisperOptions, medllama] = await Promise.all([
        tauriService.getWhisperModelOptions(),
        tauriService.getMedLlamaMetadata()
      ]);

      whisperModelOptions = whisperOptions;
      medllamaMetadata = medllama;

      // Set default medLlamaUrl if not already set
      if (!medLlamaUrl && medllama) {
        medLlamaUrl = medllama.default_url;
      }
    } catch (error) {
      console.error('Failed to load model metadata:', error);
    }
  }

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

  function getWhisperModelInfo(size: WhisperModelSize): WhisperModelMetadata | undefined {
    return whisperModelOptions.find((opt) => opt.value === size);
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
      const whisperInfo = getWhisperModelInfo(size);

      // Update preferences immediately
      const newPreferences: ModelPreferences = {
        whisper_model_size: size,
        whisper_model_url: whisperInfo?.url || '',
        whisper_model_filename: whisperInfo?.file_name || `whisper-${size}.en.gguf`,
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

  async function handleDownloadWhisperModel() {
    try {
      downloadingModel = true;
      downloadProgress = 'Downloading Whisper model...';

      const whisperInfo = getWhisperModelInfo(selectedWhisperSize);
      if (!whisperInfo) {
        throw new Error('Whisper model information not available');
      }

      await tauriService.downloadCustomModel(whisperInfo.url, whisperInfo.file_name);

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

      await tauriService.downloadCustomModel(medLlamaUrl, medllamaMetadata?.file_name || 'med_llama.gguf');

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

  function handleLogout() {
    auth.logout();
    goto('/', { replaceState: true });
  }

  function handleClearError() {
    auth.clearError();
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
          <Separator />
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

      <!-- Model Management -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center space-x-2">
            <Settings class="h-5 w-5" />
            <span>Model Management</span>
          </CardTitle>
          <CardDescription>Download, select, and manage AI models for transcription and note generation</CardDescription
          >
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
                    {#each whisperModelOptions as option (option.value)}
                      {@const isDownloaded = downloadedModels.some(
                        (m) => m.filename === `whisper-${option.value}.en.gguf`
                      )}
                      {@const isActive = preferences.whisper_model_size === (option.value as WhisperModelSize)}
                      <button
                        onclick={() => isDownloaded && handleSelectWhisperModel(option.value as WhisperModelSize)}
                        disabled={!isDownloaded}
                        class="flex w-full items-center justify-between rounded-lg border p-4 text-left transition-colors {isActive
                          ? 'border-primary bg-primary/5'
                          : 'hover:bg-muted/50'} {!isDownloaded ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'}"
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
                    <p class="mb-2 text-sm font-medium">Current URL:</p>
                    <p class="font-mono text-xs break-all text-muted-foreground">{preferences.med_llama_url}</p>
                  </div>
                </div>
              </Tabs.Content>

              <!-- Tab 2: Download Models -->
              <Tabs.Content value="download" class="space-y-6 pt-4">
                <!-- Whisper Model Download -->
                <div class="space-y-3">
                  <Label class="text-base font-semibold">Download Whisper Model</Label>
                  <p class="text-sm text-muted-foreground">
                    Choose and download a Whisper model for transcription. Larger models are more accurate but slower
                    and use more storage.
                  </p>

                  <div class="grid gap-4 md:grid-cols-2">
                    <div class="space-y-2">
                      <Label for="whisper-download-size">Model Size</Label>
                      <Select.Root type="single" bind:value={selectedWhisperSize}>
                        <Select.Trigger id="whisper-download-size" class="w-full">
                          {whisperModelOptions.find((opt) => opt.value === selectedWhisperSize)?.label ||
                            'Select model size'}
                        </Select.Trigger>
                        <Select.Content>
                          {#each whisperModelOptions as option (option.value)}
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
                    Enter a URL to download a MedLlama model for note generation. This should be a direct download link
                    to a .gguf file.
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
                      <p class="mt-2 text-xs text-muted-foreground">
                        Download models from the Download tab to get started.
                      </p>
                    </div>
                  {:else}
                    <div class="space-y-2">
                      {#each downloadedModels as model (model.filename)}
                        <div class="flex items-center justify-between rounded-lg border p-3">
                          <div class="flex-1">
                            <p class="font-mono text-sm font-medium">{model.filename}</p>
                            <p class="text-xs text-muted-foreground">{formatBytes(model.size_bytes)}</p>
                          </div>
                          <Button variant="destructive" size="sm" onclick={() => handleDeleteModel(model.filename)}>
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
