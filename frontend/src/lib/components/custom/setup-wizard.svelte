<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import * as Select from '$lib/components/ui/select';
  import { tauriService } from '$lib/tauriService';
  import type { WhisperModelSize } from '$lib/types';
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
  let currentDownloadingIndex = $state(-1);

  // User preferences
  let selectedWhisperSize: WhisperModelSize = $state('tiny');
  let medLlamaUrl = $state('https://huggingface.co/garcianacho/MedLlama-2-7B-GGUF/resolve/main/MedLlama-2-7B.q4_K_S.gguf?download=true');

  // Whisper model options
  const whisperModelOptions = [
    { value: 'tiny', label: 'Tiny (141 MB) - Fastest', size: 141 },
    { value: 'base', label: 'Base (142 MB) - Fast', size: 142 },
    { value: 'small', label: 'Small (466 MB) - Balanced', size: 466 },
    { value: 'medium', label: 'Medium (1.5 GB) - Accurate', size: 1500 },
    { value: 'large', label: 'Large (3.1 GB) - Most Accurate', size: 3100 }
  ];

  $effect(() => {
    setupProgressListener();
    loadModelPreferences();
    loadModels();
  });

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

  function buildModelsList(): ModelDownloadInfo[] {
    const whisperOption = whisperModelOptions.find(opt => opt.value === selectedWhisperSize);

    return [
      {
        name: `Whisper ${selectedWhisperSize.charAt(0).toUpperCase() + selectedWhisperSize.slice(1)}`,
        url: getWhisperModelUrl(selectedWhisperSize),
        file_name: `whisper-${selectedWhisperSize}.en.gguf`,
        size_mb: whisperOption?.size || 141
      },
      {
        name: 'Whisperfile (Runtime)',
        url: 'https://huggingface.co/Mozilla/whisperfile/resolve/main/whisper-tiny.en.llamafile',
        file_name: 'whisperfile',
        size_mb: 50
      },
      {
        name: 'Llamafile (Runtime)',
        url: 'https://github.com/Mozilla-Ocho/llamafile/releases/download/0.9.3/llamafile-0.9.3',
        file_name: 'llamafile',
        size_mb: 5
      },
      {
        name: 'MedLlama',
        url: medLlamaUrl,
        file_name: 'med_llama.gguf',
        size_mb: 3800
      }
    ];
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
      const modelIndex = models.findIndex((m) => 
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
      // Save user preferences before downloading
      await tauriService.saveModelPreferences({
        whisper_model_size: selectedWhisperSize,
        whisper_model_url: getWhisperModelUrl(selectedWhisperSize),
        whisper_model_filename: `whisper-${selectedWhisperSize}.en.gguf`,
        med_llama_url: medLlamaUrl,
        med_llama_filename: 'med_llama.gguf',
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
    const failedModels = models.filter(m => m.status === 'failed');
    if (failedModels.length === 0) return;

    isDownloading = true;

    for (const failedModel of failedModels) {
      const modelIndex = models.findIndex(m => m.model.file_name === failedModel.model.file_name);
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
    return models.some(m => m.status === 'failed');
  }

  async function handleWhisperModelChange() {
    try {
      // Save the new preference immediately
      await tauriService.saveModelPreferences({
        whisper_model_size: selectedWhisperSize,
        whisper_model_url: getWhisperModelUrl(selectedWhisperSize),
        whisper_model_filename: `whisper-${selectedWhisperSize}.en.gguf`,
        med_llama_url: medLlamaUrl,
        med_llama_filename: 'med_llama.gguf',
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
      // Save the new preference immediately
      await tauriService.saveModelPreferences({
        whisper_model_size: selectedWhisperSize,
        whisper_model_url: getWhisperModelUrl(selectedWhisperSize),
        whisper_model_filename: `whisper-${selectedWhisperSize}.en.gguf`,
        med_llama_url: medLlamaUrl,
        med_llama_filename: 'med_llama.gguf',
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
      <CardDescription>
        First-time setup: Download AI models required for the application to function.
      </CardDescription>
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
          <h3 class="text-sm font-medium">Required Models:</h3>
          {#each models as modelStatus, index}
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
                    <p class="text-xs text-muted-foreground pl-6">
                      Size: {modelStatus.model.size_mb.toFixed(0)} MB
                    </p>
                  </div>
                </div>

                <!-- Model Configuration (only show for configurable models when not downloading) -->
                {#if !isDownloading && modelStatus.status !== 'downloading'}
                  {#if modelStatus.model.name.includes('Whisper') && !modelStatus.model.name.includes('file')}
                    <!-- Whisper Model Selection -->
                    <div class="pl-6 space-y-2">
                      <Label for="whisper-size-select-{index}" class="text-xs font-medium text-muted-foreground">
                        Model Size
                      </Label>
                      <Select.Root
                        type="single"
                        bind:value={selectedWhisperSize}
                        onValueChange={handleWhisperModelChange}
                      >
                        <Select.Trigger id="whisper-size-select-{index}" class="w-full h-8 text-xs">
                          {whisperModelOptions.find(opt => opt.value === selectedWhisperSize)?.label || 'Select model size'}
                        </Select.Trigger>
                        <Select.Content>
                          {#each whisperModelOptions as option}
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
                    <div class="pl-6 space-y-2">
                      <Label for="medllama-url-input-{index}" class="text-xs font-medium text-muted-foreground">
                        Model URL
                      </Label>
                      <Input
                        id="medllama-url-input-{index}"
                        type="url"
                        bind:value={medLlamaUrl}
                        placeholder="https://huggingface.co/..."
                        class="font-mono text-xs h-8"
                        onchange={handleMedLlamaUrlChange}
                      />
                      <p class="text-xs text-muted-foreground">
                        Direct download link to a .gguf model file.
                      </p>
                    </div>
                  {/if}
                {/if}

                <!-- Download Progress -->
                {#if modelStatus.status === 'downloading'}
                  <div class="space-y-1 pl-6 pt-2">
                    <div class="flex items-center justify-between text-xs">
                      <span class="text-muted-foreground">
                        {modelStatus.downloadedMB.toFixed(1)} / {modelStatus.totalMB.toFixed(1)} MB
                      </span>
                      <span class="text-muted-foreground">{modelStatus.progress.toFixed(1)}%</span>
                    </div>
                    <Progress value={modelStatus.progress} class="h-1.5" />
                  </div>
                {:else if modelStatus.status === 'failed' && modelStatus.error}
                  <p class="text-xs text-red-600 pl-6 pt-1">{modelStatus.error}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center justify-between border-t pt-4">
          <div class="text-sm text-muted-foreground text-balance">
            {#if isDownloading}
              Downloading models... This may take several minutes.
            {:else if allModelsDownloaded()}
              All models downloaded successfully!
            {:else if hasFailedDownloads()}
              Some downloads failed. You can retry failed downloads or continue with available models.
            {:else}
              Configure your preferred models above, then click "Start Download" to begin setup.
            {/if}
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
                Start Download
              {/if}
            </Button>
          </div>
        </div>
      {:else}
        <!-- Setup Complete -->
        <div class="space-y-4 text-center py-8">
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
