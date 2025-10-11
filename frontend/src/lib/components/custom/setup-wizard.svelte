<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { tauriService } from '$lib/tauriService';
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

  $effect(() => {
    loadModels();
    setupProgressListener();
  });

  async function loadModels() {
    try {
      const requiredModels = await tauriService.getRequiredModelsList();
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

      // Find the model by matching the name
      const modelIndex = models.findIndex((m) =>
        progress.file_name.includes(m.model.name) || m.model.name.includes(progress.file_name)
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

    for (let i = 0; i < models.length; i++) {
      if (models[i].status === 'completed') {
        continue;
      }

      currentDownloadingIndex = i;
      models[i].status = 'downloading';
      models[i].progress = 0;

      try {
        await tauriService.downloadModelFile(models[i].model);
        models[i].status = 'completed';
        models[i].progress = 100;
      } catch (error) {
        console.error(`Failed to download ${models[i].model.name}:`, error);
        models[i].status = 'failed';
        models[i].error = error instanceof Error ? error.message : String(error);
        isDownloading = false;
        currentDownloadingIndex = -1;
        return;
      }
    }

    currentDownloadingIndex = -1;
    isDownloading = false;

    // Mark setup as complete
    try {
      await tauriService.completeSetup();
      setupComplete = true;
    } catch (error) {
      console.error('Failed to mark setup as complete:', error);
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
                download AI models (~4.1 GB total). This is a one-time setup.
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
            </div>
          {/each}
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center justify-between border-t pt-4">
          <div class="text-sm text-muted-foreground">
            {#if isDownloading}
              Downloading models... This may take several minutes.
            {:else if allModelsDownloaded()}
              All models downloaded successfully!
            {:else}
              Click "Start Download" to begin setup.
            {/if}
          </div>
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
