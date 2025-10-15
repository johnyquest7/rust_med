<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import * as Select from '$lib/components/ui/select';
  import { tauriService } from '$lib/tauriService';
  import type {
    WhisperModelSize,
    WhisperModelMetadata,
    RuntimeBinaryMetadata,
    MedLlamaModelMetadata
  } from '$lib/types';
  import Download from '@lucide/svelte/icons/download';
  import CheckCircle from '@lucide/svelte/icons/check-circle';
  import XCircle from '@lucide/svelte/icons/x-circle';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import AlertCircle from '@lucide/svelte/icons/alert-circle';

  interface ModelDownloadInfo {
    name: string;
    url: string;
    file_name: string;
    size_mb: number;
  }

  interface DownloadProgress {
    file_name: string;
    downloaded_bytes: number;
    total_bytes: number | null;
    percentage: number;
    status: 'downloading' | 'completed' | 'failed';
  }

  interface ModelStatus {
    model: ModelDownloadInfo;
    status: 'pending' | 'downloading' | 'completed' | 'failed';
    progress: number;
    downloadedMB: number;
    totalMB: number;
    error?: string;
  }

  let models = $state<ModelStatus[]>([]);
  let isDownloading = $state(false);
  let setupComplete = $state(false);

  // User preferences
  let selectedWhisperSize: WhisperModelSize = $state('tiny');
  let medLlamaUrl = $state('');

  // Model metadata from backend (SINGLE SOURCE OF TRUTH)
  let whisperModelOptions = $state<WhisperModelMetadata[]>([]);
  let runtimeBinaries = $state<RuntimeBinaryMetadata[]>([]);
  let medllamaMetadata = $state<MedLlamaModelMetadata | null>(null);

  $effect(() => {
    setupProgressListener();
    loadModelMetadata();
    loadModelPreferences();
    loadModels();
  });

  async function loadModelMetadata() {
    try {
      // Load all metadata from backend (SINGLE SOURCE OF TRUTH)
      const [whisperOptions, binaries, medllama] = await Promise.all([
        tauriService.getWhisperModelOptions(),
        tauriService.getRuntimeBinaries(),
        tauriService.getMedLlamaMetadata()
      ]);

      whisperModelOptions = whisperOptions;
      runtimeBinaries = binaries;
      medllamaMetadata = medllama;

      // Set default medLlamaUrl if not already set
      if (!medLlamaUrl && medllama) {
        medLlamaUrl = medllama.default_url;
      }
    } catch (error) {
      console.error('Failed to load model metadata:', error);
    }
  }

  async function loadModelPreferences() {
    try {
      const preferences = await tauriService.getModelPreferences();
      selectedWhisperSize = preferences.whisper_model_size;
      medLlamaUrl = preferences.med_llama_url;
    } catch (error) {
      console.error('Failed to load model preferences:', error);
      // Use defaults if preferences can't be loaded
    }
  }

  function getWhisperModelInfo(size: WhisperModelSize): WhisperModelMetadata | undefined {
    return whisperModelOptions.find((opt) => opt.value === size);
  }

  function buildModelsList(): ModelDownloadInfo[] {
    const whisperOption = getWhisperModelInfo(selectedWhisperSize);
    const models: ModelDownloadInfo[] = [];

    // Add runtime binaries
    for (const binary of runtimeBinaries) {
      models.push({
        name: binary.name,
        url: binary.url,
        file_name: binary.file_name,
        size_mb: binary.size_mb
      });
    }

    // Add selected Whisper model
    if (whisperOption) {
      models.push({
        name: `Whisper ${selectedWhisperSize.charAt(0).toUpperCase() + selectedWhisperSize.slice(1)}`,
        url: whisperOption.url,
        file_name: whisperOption.file_name,
        size_mb: whisperOption.size
      });
    }

    // Add MedLlama model
    if (medllamaMetadata) {
      models.push({
        name: medllamaMetadata.name,
        url: medLlamaUrl || medllamaMetadata.default_url,
        file_name: medllamaMetadata.file_name,
        size_mb: medllamaMetadata.size_mb
      });
    }

    return models;
  }

  async function loadModels() {
    try {
      const requiredModels = buildModelsList();
      const modelStatuses = await tauriService.checkModelsDownloaded();

      models = requiredModels.map((model) => {
        const status = modelStatuses.find((s) => s[0].file_name === model.file_name);
        return {
          model,
          status: status && status[1] ? 'completed' : 'pending',
          progress: status && status[1] ? 100 : 0,
          downloadedMB: 0,
          totalMB: model.size_mb
        };
      });
    } catch (error) {
      console.error('Failed to load models:', error);
    }
  }

  async function setupProgressListener() {
    await tauriService.listen<DownloadProgress>('download-progress', (event) => {
      const progress = event.payload;

      // Find the model by matching the file_name from progress with model.file_name
      const modelIndex = models.findIndex(
        (m) =>
          progress.file_name === m.model.file_name ||
          progress.file_name.includes(m.model.file_name) ||
          m.model.file_name.includes(progress.file_name)
      );

      if (modelIndex !== -1) {
        const downloadedMB = progress.downloaded_bytes / (1024 * 1024);
        const totalMB = progress.total_bytes ? progress.total_bytes / (1024 * 1024) : models[modelIndex].totalMB;

        models[modelIndex] = {
          ...models[modelIndex],
          status: progress.status === 'completed' ? 'completed' : 'downloading',
          progress: progress.percentage,
          downloadedMB,
          totalMB
        };
      }
    });
  }

  async function downloadAllModels() {
    isDownloading = true;

    try {
      // Get whisper model info from metadata
      const whisperInfo = getWhisperModelInfo(selectedWhisperSize);

      // Save user preferences before downloading
      await tauriService.saveModelPreferences({
        whisper_model_size: selectedWhisperSize,
        whisper_model_url: whisperInfo?.url || '',
        whisper_model_filename: whisperInfo?.file_name || `whisper-${selectedWhisperSize}.en.gguf`,
        med_llama_url: medLlamaUrl,
        med_llama_filename: medllamaMetadata?.file_name || 'med_llama.gguf',
        updated_at: new Date().toISOString()
      });
    } catch (error) {
      console.error('Failed to save preferences:', error);
    }

    // Reload models based on user selection
    await loadModels();

    for (let i = 0; i < models.length; i++) {
      if (models[i].status === 'completed') {
        continue;
      }

      currentDownloadingIndex = i;
      models[i].status = 'downloading';
      models[i].progress = 0;
      models[i].error = undefined;

      try {
        await tauriService.downloadModelFile(models[i].model);
        models[i].status = 'completed';
        models[i].progress = 100;
      } catch (error) {
        console.error(`Failed to download ${models[i].model.name}:`, error);
        models[i].status = 'failed';
        models[i].error = error instanceof Error ? error.message : String(error);
        // Continue with other models instead of stopping completely
      }
    }

    currentDownloadingIndex = -1;
    isDownloading = false;

    // Mark setup as complete only if all models downloaded successfully
    if (allModelsDownloaded()) {
      try {
        await tauriService.completeSetup();
        setupComplete = true;
      } catch (error) {
        console.error('Failed to mark setup as complete:', error);
      }
    }
  }

  function getTotalSize() {
    return models.reduce((sum, m) => sum + m.model.size_mb, 0);
  }

  function getDownloadedSize() {
    return models.reduce((sum, m) => {
      if (m.status === 'completed') return sum + m.model.size_mb;
      if (m.status === 'downloading') return sum + m.downloadedMB;
      return sum;
    }, 0);
  }

  function getOverallProgress() {
    const total = getTotalSize();
    const downloaded = getDownloadedSize();
    return total > 0 ? (downloaded / total) * 100 : 0;
  }

  function allModelsDownloaded() {
    return models.every((m) => m.status === 'completed');
  }

  function handleComplete() {
    window.location.reload();
  }

  async function retryFailedDownloads() {
    const failedModels = models.filter((m) => m.status === 'failed');
    if (failedModels.length === 0) return;

    isDownloading = true;

    for (const failedModel of failedModels) {
      const modelIndex = models.findIndex((m) => m.model.file_name === failedModel.model.file_name);
      if (modelIndex === -1) continue;

      currentDownloadingIndex = modelIndex;
      models[modelIndex].status = 'downloading';
      models[modelIndex].progress = 0;
      models[modelIndex].error = undefined;

      try {
        await tauriService.downloadModelFile(models[modelIndex].model);
        models[modelIndex].status = 'completed';
        models[modelIndex].progress = 100;
      } catch (error) {
        console.error(`Failed to retry download ${models[modelIndex].model.name}:`, error);
        models[modelIndex].status = 'failed';
        models[modelIndex].error = error instanceof Error ? error.message : String(error);
      }
    }

    currentDownloadingIndex = -1;
    isDownloading = false;

    // Mark setup as complete if all models are now downloaded
    if (allModelsDownloaded()) {
      try {
        await tauriService.completeSetup();
        setupComplete = true;
      } catch (error) {
        console.error('Failed to mark setup as complete:', error);
      }
    }
  }

  function hasFailedDownloads() {
    return models.some((m) => m.status === 'failed');
  }

  async function handleWhisperModelChange() {
    try {
      // Get whisper model info from metadata
      const whisperInfo = getWhisperModelInfo(selectedWhisperSize);

      // Save the new preference immediately
      await tauriService.saveModelPreferences({
        whisper_model_size: selectedWhisperSize,
        whisper_model_url: whisperInfo?.url || '',
        whisper_model_filename: whisperInfo?.file_name || `whisper-${selectedWhisperSize}.en.gguf`,
        med_llama_url: medLlamaUrl,
        med_llama_filename: medllamaMetadata?.file_name || 'med_llama.gguf',
        updated_at: new Date().toISOString()
      });

      // Reload models with the new selection
      await loadModels();
    } catch (error) {
      console.error('Failed to save whisper model preference:', error);
    }
  }

  async function handleMedLlamaUrlChange() {
    try {
      // Get whisper model info from metadata
      const whisperInfo = getWhisperModelInfo(selectedWhisperSize);

      // Save the new preference immediately
      await tauriService.saveModelPreferences({
        whisper_model_size: selectedWhisperSize,
        whisper_model_url: whisperInfo?.url || '',
        whisper_model_filename: whisperInfo?.file_name || `whisper-${selectedWhisperSize}.en.gguf`,
        med_llama_url: medLlamaUrl,
        med_llama_filename: medllamaMetadata?.file_name || 'med_llama.gguf',
        updated_at: new Date().toISOString()
      });

      // Reload models with the new selection
      await loadModels();
    } catch (error) {
      console.error('Failed to save medllama url preference:', error);
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center bg-background px-4">
  <Card class="w-full max-w-2xl">
    <CardHeader>
      <CardTitle class="text-2xl">Welcome to Medical Note Generator</CardTitle>
      <CardDescription>First-time setup: Download AI models required for the application to function.</CardDescription>
    </CardHeader>

    <CardContent class="space-y-6">
      {#if !setupComplete}
        <!-- Info Banner -->
        <div class="rounded-lg border border-blue-200 bg-blue-50 p-4">
          <div class="flex items-start space-x-3">
            <AlertCircle class="mt-0.5 h-5 w-5 text-blue-600" />
            <div class="flex-1 space-y-1">
              <p class="text-sm font-medium text-blue-900">About this setup</p>
              <p class="text-sm text-blue-700">
                This app processes all data locally on your device for maximum privacy. To enable this, we need to
                download AI models. This is a one-time setup.
              </p>
            </div>
          </div>
        </div>

        <!-- Overall Progress -->
        {#if isDownloading}
          <div class="space-y-2">
            <div class="flex items-center justify-between text-sm">
              <span class="font-medium">Overall Progress</span>
              <span class="text-muted-foreground">
                {(getDownloadedSize() / 1024).toFixed(2)} / {(getTotalSize() / 1024).toFixed(2)} GB
              </span>
            </div>
            <Progress value={getOverallProgress()} class="h-2" />
          </div>
        {/if}

        <!-- Models List -->
        <div class="space-y-3">
          <h3 class="text-sm font-medium">AI Models</h3>
          {#each models as modelStatus, index (modelStatus.name)}
            <div class="rounded-lg border p-4">
              <div class="space-y-3">
                <!-- Model Header -->
                <div class="flex items-start justify-between">
                  <div class="flex-1 space-y-1">
                    <div class="flex items-center space-x-2">
                      {#if modelStatus.status === 'completed'}
                        <CheckCircle class="h-4 w-4 text-green-600" />
                      {:else if modelStatus.status === 'downloading'}
                        <Loader2 class="h-4 w-4 animate-spin text-blue-600" />
                      {:else if modelStatus.status === 'failed'}
                        <XCircle class="h-4 w-4 text-red-600" />
                      {:else}
                        <div class="h-4 w-4 rounded-full border-2 border-muted-foreground/30"></div>
                      {/if}
                      <p class="text-sm font-medium">{modelStatus.model.name}</p>
                    </div>
                    <p class="pl-6 text-xs text-muted-foreground">
                      Size: {modelStatus.model.size_mb.toFixed(0)} MB
                    </p>
                  </div>
                </div>

                <!-- Model Configuration (only show for configurable models when not downloading) -->
                {#if !isDownloading && modelStatus.status !== 'downloading'}
                  {#if modelStatus.model.name.includes('Whisper') && !modelStatus.model.name.includes('file')}
                    <!-- Whisper Model Selection -->
                    <div class="space-y-2 pl-6">
                      <Label for="whisper-size-select-{index}" class="text-xs font-medium text-muted-foreground">
                        Model Size
                      </Label>
                      <Select.Root
                        type="single"
                        bind:value={selectedWhisperSize}
                        onValueChange={handleWhisperModelChange}
                      >
                        <Select.Trigger id="whisper-size-select-{index}" class="h-8 w-full text-xs">
                          {whisperModelOptions.find((opt) => opt.value === selectedWhisperSize)?.label ||
                            'Select model size'}
                        </Select.Trigger>
                        <Select.Content>
                          {#each whisperModelOptions as option (option.value)}
                            <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
                          {/each}
                        </Select.Content>
                      </Select.Root>
                      <p class="text-xs text-muted-foreground">
                        Larger models are more accurate but slower and use more storage.
                      </p>
                    </div>
                  {:else if modelStatus.model.name.includes('MedLlama')}
                    <!-- MedLlama URL Input -->
                    <div class="space-y-2 pl-6">
                      <Label for="medllama-url-input-{index}" class="text-xs font-medium text-muted-foreground">
                        Model URL
                      </Label>
                      <Input
                        id="medllama-url-input-{index}"
                        type="url"
                        bind:value={medLlamaUrl}
                        placeholder="https://huggingface.co/..."
                        class="text-xs"
                        onchange={handleMedLlamaUrlChange}
                      />
                      <p class="text-xs text-muted-foreground">Direct download link to a .gguf model file.</p>
                    </div>
                  {/if}
                {/if}

                <!-- Download Progress -->
                {#if modelStatus.status === 'downloading'}
                  <div class="space-y-1 pt-2 pl-6">
                    <div class="flex items-center justify-between text-xs">
                      <span class="text-muted-foreground">
                        {modelStatus.downloadedMB.toFixed(1)} / {modelStatus.totalMB.toFixed(1)} MB
                      </span>
                      <span class="text-muted-foreground">{modelStatus.progress.toFixed(1)}%</span>
                    </div>
                    <Progress value={modelStatus.progress} class="h-1.5" />
                  </div>
                {:else if modelStatus.status === 'failed' && modelStatus.error}
                  <p class="pt-1 pl-6 text-xs text-red-600">{modelStatus.error}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center justify-between pt-4">
          <div class="text-sm text-balance text-muted-foreground">
            {#if isDownloading}
              Downloading models... This may take several minutes.
            {:else if allModelsDownloaded()}
              All models downloaded successfully!
            {:else if hasFailedDownloads()}
              Some downloads failed. You can retry failed downloads or continue with available models.
            {:else}{/if}
          </div>
          <div class="flex gap-2">
            {#if hasFailedDownloads() && !isDownloading}
              <Button variant="outline" onclick={retryFailedDownloads}>
                <Download class="mr-2 h-4 w-4" />
                Retry Failed
              </Button>
            {/if}
            <Button onclick={downloadAllModels} disabled={isDownloading || allModelsDownloaded()}>
              {#if isDownloading}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                Downloading...
              {:else if allModelsDownloaded()}
                <CheckCircle class="mr-2 h-4 w-4" />
                Complete
              {:else}
                <Download class="mr-2 h-4 w-4" />
                Download Models
              {/if}
            </Button>
          </div>
        </div>
      {:else}
        <!-- Setup Complete -->
        <div class="space-y-4 py-8 text-center">
          <CheckCircle class="mx-auto h-16 w-16 text-green-600" />
          <div class="space-y-2">
            <h3 class="text-xl font-semibold">Setup Complete!</h3>
            <p class="text-sm text-muted-foreground">All AI models have been downloaded successfully.</p>
          </div>
          <Button onclick={handleComplete} class="mt-4">Continue to App</Button>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
