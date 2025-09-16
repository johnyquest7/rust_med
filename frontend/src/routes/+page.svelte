<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Separator } from '$lib/components/ui/separator';
  import * as Select from '$lib/components/ui/select';
  import { tauriService } from '$lib/tauriService';
  import type { RecordingState } from '$lib/types';
  import { Mic, Play, Pause, Square, Loader2, Star, AlertCircle, CheckCircle, AlertTriangle } from 'lucide-svelte';

  // TODO: handle the processing UI and post-processing UI

  // Form state
  let formData = $state({
    firstName: '',
    lastName: '',
    dateOfBirth: new Date().toISOString().split('T')[0],
    noteType: 'soap' as 'soap' | 'full'
  });

  let errors = $state<Record<string, string>>({});

  // Recording state - moved from global state to component level
  let recordingState = $state<RecordingState>('not-ready');
  let recordingTime = $state(0);
  let availableMicrophones = $state<MediaDeviceInfo[]>([]);
  let selectedMicrophoneId = $state('');
  let isMicrophoneConnected = $state(false);

  // Consolidated status and error management
  let statusType = $state<'info' | 'success' | 'warning' | 'error'>('info');
  let microphoneError = $state('');
  let recordingError = $state('');
  let processingError = $state('');

  // Audio recording state
  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];
  let stream: MediaStream | null = null;
  let recordingTimerId: number | null = null;

  // Computed recording state
  let isRecording = $derived(recordingState === 'recording');
  let isPaused = $derived(recordingState === 'paused');
  let canPauseResume = $derived(isRecording || isPaused);
  let needsMicrophoneConnection = $derived(!isMicrophoneConnected);

  // Connect microphone and get available devices
  async function connectMicrophone() {
    try {
      // Request microphone permission first to get proper device IDs
      await navigator.mediaDevices.getUserMedia({ audio: true });

      // Get available microphones with proper device IDs
      const devices = await navigator.mediaDevices.enumerateDevices();
      availableMicrophones = devices.filter((device) => device.kind === 'audioinput');

      if (availableMicrophones.length > 0) {
        selectedMicrophoneId = availableMicrophones[0].deviceId;
        isMicrophoneConnected = true;
        recordingState = 'ready';
        console.log('Available microphones:', availableMicrophones);
      } else {
        throw new Error('No microphones found');
      }
    } catch (error) {
      console.error('Failed to connect microphone:', error);
      microphoneError = `Failed to connect microphone: ${error instanceof Error ? error.message : 'Unknown error'}`;
      setTimeout(() => (microphoneError = ''), 5000);
    }
  }

  function validateForm() {
    errors = {};

    if (!formData.firstName.trim()) {
      errors.firstName = 'First name is required';
    }

    if (!formData.lastName.trim()) {
      errors.lastName = 'Last name is required';
    }

    if (!formData.dateOfBirth) {
      errors.dateOfBirth = 'Date of birth is required';
    } else {
      const birthDate = new Date(formData.dateOfBirth);
      const today = new Date();
      if (birthDate > today) {
        errors.dateOfBirth = 'Date of birth cannot be in the future';
      }
    }

    if (!isMicrophoneConnected) {
      errors.microphone = 'Microphone must be connected to record';
    }

    return Object.keys(errors).length === 0;
  }

  // Recording functions - moved from AudioService
  async function startRecording(deviceId?: string): Promise<void> {
    try {
      const audioConstraints = {
        deviceId: deviceId ? { exact: deviceId } : undefined,
        sampleRate: 44100,
        channelCount: 1,
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
        latency: 0.01,
        volume: 1.0
      };

      stream = await navigator.mediaDevices.getUserMedia({
        audio: audioConstraints
      });

      const supportedFormats = [
        'audio/webm;codecs=opus',
        'audio/webm',
        'audio/ogg;codecs=opus',
        'audio/ogg',
        'audio/wav',
        'audio/wave',
        'audio/x-wav',
        'audio/mpeg',
        'audio/mp3',
        'audio/mp4',
        'audio/aac',
        'audio/flac'
      ];

      let selectedFormat = null;
      for (const mimeType of supportedFormats) {
        if (MediaRecorder.isTypeSupported(mimeType)) {
          selectedFormat = { mime: mimeType, needsConversion: true };
          break;
        }
      }

      if (!selectedFormat) {
        throw new Error('No audio recording formats supported by your browser');
      }

      mediaRecorder = new MediaRecorder(stream, {
        mimeType: selectedFormat.mime,
        audioBitsPerSecond: 128000
      });

      audioChunks = [];
      recordingState = 'recording';
      const recordingFrequency = 1000; // Collect data every 1 second

      return new Promise((resolve, reject) => {
        mediaRecorder!.ondataavailable = (event) => {
          if (event.data.size > 0) {
            audioChunks.push(event.data);
          }
        };

        mediaRecorder!.onstop = () => {
          if (recordingState !== 'paused') {
            recordingState = 'stopped';
            resolve();
          }
        };

        mediaRecorder!.onerror = (event) => {
          reject(new Error(`Recording error: ${event.error?.message || 'Unknown error'}`));
        };

        mediaRecorder!.start(recordingFrequency); // Collect data every second
        resolve();
      });
    } catch (error) {
      throw new Error(`Failed to start recording: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  function pauseRecording(): void {
    if (!mediaRecorder) {
      throw new Error('No active recording');
    }

    try {
      mediaRecorder.pause();
      recordingState = 'paused';
    } catch (error) {
      throw new Error(`Failed to pause recording: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  function resumeRecording(): void {
    if (!mediaRecorder) {
      throw new Error('No active recording');
    }

    try {
      mediaRecorder.resume();
      recordingState = 'recording';
    } catch (error) {
      throw new Error(`Failed to resume recording: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  function stopRecording(): void {
    if (!mediaRecorder) {
      throw new Error('No active recording');
    }

    try {
      mediaRecorder.stop();
      recordingState = 'stopped';
    } catch (error) {
      throw new Error(`Failed to stop recording: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async function getRecordedAudio(): Promise<Blob> {
    if (audioChunks.length === 0) {
      throw new Error('No audio data recorded');
    }

    // Create blob from recorded chunks
    const recordedBlob = new Blob(audioChunks, {
      type: mediaRecorder?.mimeType || 'audio/webm'
    });

    // Convert to WAV format for better compatibility with transcription services
    return await convertToWav(recordedBlob);
  }

  // Audio conversion functions
  async function convertToWav(audioBlob: Blob): Promise<Blob> {
    try {
      // Use OfflineAudioContext for better performance
      const audioContext = new window.AudioContext({
        sampleRate: 16000 // Whisper prefers 16kHz
      });

      // Read the audio blob as array buffer
      const arrayBuffer = await audioBlob.arrayBuffer();

      // Decode the audio data
      const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);

      // Convert to WAV format
      const wavBlob = audioBufferToWav(audioBuffer);

      // Clean up audio context
      audioContext.close();

      return wavBlob;
    } catch (error) {
      console.error('Audio conversion failed:', error);
      // Return original blob if conversion fails
      return audioBlob;
    }
  }

  function audioBufferToWav(audioBuffer: AudioBuffer): Blob {
    const targetSampleRate = 16000;
    const numberOfChannels = 1;
    const bitDepth = 16;
    const bytesPerSample = bitDepth / 8;
    const blockAlign = numberOfChannels * bytesPerSample;

    // Get channel data and convert to mono if needed
    let samples;
    if (audioBuffer.numberOfChannels === 1) {
      samples = audioBuffer.getChannelData(0);
    } else {
      // Mix stereo to mono
      const left = audioBuffer.getChannelData(0);
      const right = audioBuffer.getChannelData(1);
      samples = new Float32Array(left.length);

      for (let i = 0; i < left.length; i++) {
        samples[i] = (left[i] + right[i]) / 2;
      }
    }

    // Resample if needed (simple linear interpolation)
    if (audioBuffer.sampleRate !== targetSampleRate) {
      samples = resampleAudio(samples, audioBuffer.sampleRate, targetSampleRate);
    }

    const length = samples.length;
    const arrayBuffer = new ArrayBuffer(44 + length * bytesPerSample);
    const view = new DataView(arrayBuffer);

    // Write WAV header
    writeWavHeader(view, length, targetSampleRate, numberOfChannels, bitDepth, bytesPerSample, blockAlign);

    // Convert and write samples
    writeSamples(view, samples, 44);

    return new Blob([arrayBuffer], { type: 'audio/wav' });
  }

  function resampleAudio(samples: Float32Array, originalSampleRate: number, targetSampleRate: number): Float32Array {
    const ratio = originalSampleRate / targetSampleRate;
    const newLength = Math.round(samples.length / ratio);
    const resampled = new Float32Array(newLength);

    for (let i = 0; i < newLength; i++) {
      const originalIndex = i * ratio;
      const index1 = Math.floor(originalIndex);
      const index2 = Math.min(index1 + 1, samples.length - 1);
      const fraction = originalIndex - index1;

      resampled[i] = samples[index1] * (1 - fraction) + samples[index2] * fraction;
    }

    return resampled;
  }

  function writeWavHeader(
    view: DataView,
    length: number,
    sampleRate: number,
    channels: number,
    bitDepth: number,
    bytesPerSample: number,
    blockAlign: number
  ) {
    const writeString = (offset: number, string: string) => {
      for (let i = 0; i < string.length; i++) {
        view.setUint8(offset + i, string.charCodeAt(i));
      }
    };

    // RIFF chunk descriptor
    writeString(0, 'RIFF');
    view.setUint32(4, 36 + length * bytesPerSample, true);
    writeString(8, 'WAVE');

    // fmt sub-chunk
    writeString(12, 'fmt ');
    view.setUint32(16, 16, true); // Subchunk1Size
    view.setUint16(20, 1, true); // AudioFormat (PCM)
    view.setUint16(22, channels, true); // NumChannels
    view.setUint32(24, sampleRate, true); // SampleRate
    view.setUint32(28, sampleRate * channels * bytesPerSample, true); // ByteRate
    view.setUint16(32, blockAlign, true); // BlockAlign
    view.setUint16(34, bitDepth, true); // BitsPerSample

    // data sub-chunk
    writeString(36, 'data');
    view.setUint32(40, length * bytesPerSample, true);
  }

  function writeSamples(view: DataView, samples: Float32Array, offset: number) {
    for (let i = 0; i < samples.length; i++) {
      const sample = Math.max(-1, Math.min(1, samples[i]));
      const intSample = Math.round(sample * 32767);
      view.setInt16(offset + i * 2, intSample, true);
    }
  }

  // Timer functions
  function startTimer() {
    if (recordingTimerId) {
      clearInterval(recordingTimerId);
    }

    recordingTimerId = setInterval(() => {
      recordingTime = recordingTime + 1;
    }, 1000);
  }

  function stopTimer() {
    if (recordingTimerId) {
      clearInterval(recordingTimerId);
      recordingTimerId = null;
    }
  }

  // Event handlers
  async function handleConnectMicrophone() {
    try {
      await connectMicrophone();
    } catch (error) {
      console.error('Failed to connect microphone:', error);
      microphoneError = `Failed to connect microphone: ${error instanceof Error ? error.message : 'Unknown error'}`;
      setTimeout(() => (microphoneError = ''), 5000);
    }
  }

  async function handleRecord() {
    if (!validateForm()) {
      return;
    }

    try {
      statusType = 'info';
      await startRecording(selectedMicrophoneId);
      recordingState = 'recording';
      recordingTime = 0;
      startTimer();
      statusType = 'info';
    } catch (error) {
      console.error('Failed to start recording:', error);
      recordingError = `Failed to start recording: ${error instanceof Error ? error.message : 'Unknown error'}`;
      setTimeout(() => (recordingError = ''), 5000);
    }
  }

  async function handlePauseResume() {
    try {
      if (recordingState === 'paused') {
        // Resume recording
        statusType = 'info';
        resumeRecording();
        recordingState = 'recording';
        startTimer();
        statusType = 'info';
      } else {
        // Pause recording
        statusType = 'info';
        pauseRecording();
        recordingState = 'paused';
        stopTimer();
        statusType = 'warning';
      }
    } catch (error) {
      console.error('Error in pauseResumeRecording:', error);
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      recordingError = `Failed to pause/resume recording: ${errorMsg}`;
      setTimeout(() => (recordingError = ''), 5000);
    }
  }

  async function handleStopRecording() {
    try {
      statusType = 'info';
      stopRecording();
      recordingState = 'stopped';
      stopTimer();
      statusType = 'success';
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      recordingError = `Failed to stop recording: ${errorMsg}`;
      setTimeout(() => (recordingError = ''), 5000);
      recordingState = 'not-ready';
    }

    // Start the processing
    processRecording();
  }

  async function processRecording() {
    if (!validateForm()) {
      return;
    }

    try {
      statusType = 'info';

      // Get the recorded audio blob
      const audioBlob = await getRecordedAudio();

      if (!audioBlob) {
        throw new Error('No audio data recorded');
      }

      statusType = 'info';

      // Write the audio to a file using Tauri service
      const appDataDir = await tauriService.appLocalDataDir();
      const audioFilename = 'debug.wav';
      const audioPath = await tauriService.joinPath(appDataDir, audioFilename);

      statusType = 'info';
      console.log('Transcribing audio...');
      const transcriptionResult = await tauriService.transcribeAudio(audioPath);
      if (!transcriptionResult.success) {
        console.error('Transcription failed:', transcriptionResult.error);
        throw new Error(transcriptionResult.error || 'Transcription failed');
      }

      const transcript = transcriptionResult.transcript;
      statusType = 'info';

      const noteGenResult = await tauriService.generateMedicalNote(transcript, formData.noteType);

      if (!noteGenResult.success) {
        throw new Error(noteGenResult.error || 'Failed to generate medical note');
      }
      const medicalNote = noteGenResult.note;

      statusType = 'success';

      // Create the note with the processed data
      const createResult = await tauriService.createNote({
        firstName: formData.firstName,
        lastName: formData.lastName,
        dateOfBirth: formData.dateOfBirth,
        noteType: formData.noteType,
        transcript: transcript,
        medicalNote: medicalNote
      });

      if (!createResult.success) {
        throw new Error(createResult.error || 'Failed to create note');
      }

      // Reset form after successful processing
      formData = {
        firstName: '',
        lastName: '',
        dateOfBirth: '',
        noteType: 'soap'
      };

      // Reset recording state
      resetRecording();
    } catch (error) {
      console.error('Failed to process recording:', error);
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      processingError = `Failed to process recording: ${errorMsg}`;
      setTimeout(() => (processingError = ''), 5000);
    }
  }

  // Reset recording state
  function resetRecording() {
    if (stream) {
      stream.getTracks().forEach((track) => track.stop());
    }
    mediaRecorder = null;
    audioChunks = [];
    stream = null;
    recordingState = 'ready';
    recordingTime = 0;
    stopTimer();
    clearStatusMessages();
    statusType = 'info';
    // Note: isInitialized and isMicrophoneConnected remain true after first setup
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  // Helper function to clear status messages
  function clearStatusMessages() {
    microphoneError = '';
    recordingError = '';
    processingError = '';
  }
</script>

<svelte:head>
  <title>Record Medical Note - Medical Note Generator</title>
  <meta name="description" content="Record a new medical note for a patient" />
</svelte:head>

<div class="container mx-auto max-w-2xl px-4 py-8">
  <Card>
    <CardHeader>
      <CardTitle class="text-2xl font-bold">Record Medical Note</CardTitle>
      <CardDescription>Enter patient information and select the type of note to generate.</CardDescription>
    </CardHeader>

    <CardContent class="space-y-6">
      <!-- Patient Information Section -->
      <div class="space-y-4">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div class="space-y-2">
            <Label for="firstName" class="text-sm font-medium">First Name*</Label>
            <Input
              id="firstName"
              type="text"
              placeholder="Enter first name"
              bind:value={formData.firstName}
              class={errors.firstName ? 'border-destructive' : ''}
              aria-invalid={!!errors.firstName}
              aria-describedby={errors.firstName ? 'firstName-error' : undefined}
              required
            />
            {#if errors.firstName}
              <p class="text-sm text-destructive">{errors.firstName}</p>
            {/if}
          </div>

          <div class="space-y-2">
            <Label for="lastName" class="text-sm font-medium">Last Name*</Label>
            <Input
              id="lastName"
              type="text"
              placeholder="Enter last name"
              bind:value={formData.lastName}
              class={errors.lastName ? 'border-destructive' : ''}
              aria-invalid={!!errors.lastName}
              aria-describedby={errors.lastName ? 'lastName-error' : undefined}
              required
            />
            {#if errors.lastName}
              <p class="text-sm text-destructive">{errors.lastName}</p>
            {/if}
          </div>
        </div>

        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div class="space-y-2">
            <Label for="dateOfBirth" class="text-sm font-medium">Date of Birth*</Label>
            <Input
              id="dateOfBirth"
              type="date"
              bind:value={formData.dateOfBirth}
              class={errors.dateOfBirth ? 'border-destructive' : ''}
              aria-invalid={!!errors.dateOfBirth}
              aria-describedby={errors.dateOfBirth ? 'dateOfBirth-error' : undefined}
              required
            />
            {#if errors.dateOfBirth}
              <p class="text-sm text-destructive">{errors.dateOfBirth}</p>
            {/if}
          </div>
        </div>

        <!-- Recording Configuration Section -->
        <!-- Note Type Selection -->
        <div class="space-y-2">
          <Label for="noteType" class="text-sm font-medium">Note Type*</Label>
          <Select.Root type="single" bind:value={formData.noteType}>
            <Select.Trigger class="w-full">
              {formData.noteType === 'soap'
                ? 'SOAP Note'
                : formData.noteType === 'full'
                  ? 'Full Note'
                  : 'Select note type'}
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="soap">
                <div class="flex flex-col">
                  <div class="text-sm font-medium">SOAP Note</div>
                  <div class="text-xs text-muted-foreground">
                    Structured format with Subjective, Objective, Assessment, and Plan sections
                  </div>
                </div>
              </Select.Item>
              <Select.Item value="full">
                <div class="flex flex-col">
                  <div class="text-sm font-medium">Full Note</div>
                  <div class="text-xs text-muted-foreground">
                    Comprehensive narrative note with detailed documentation
                  </div>
                </div>
              </Select.Item>
            </Select.Content>
          </Select.Root>
        </div>

        <!-- Microphone Setup -->
        <div class="space-y-2">
          <Label class="text-sm font-medium">Microphone*</Label>
          {#if needsMicrophoneConnection}
            <div class="space-y-3">
              <div class="rounded-lg border border-dashed border-muted-foreground/25 p-4 text-center">
                <Mic class="mx-auto mb-2 h-8 w-8 text-muted-foreground/50" />
                <p class="mb-3 text-sm text-muted-foreground">No microphone connected</p>
                <Button onclick={handleConnectMicrophone} class="w-full md:w-auto">
                  <Mic class="mr-2 h-4 w-4" />
                  Connect Microphone
                </Button>
              </div>
              {#if errors.microphone}
                <p class="text-sm text-destructive">{errors.microphone}</p>
              {/if}
              {#if microphoneError}
                <p class="text-sm text-destructive">{microphoneError}</p>
              {/if}
            </div>
          {:else if availableMicrophones.length > 0}
            <div class="space-y-2">
              <div class="rounded-lg border border-muted-foreground/25 bg-muted/10 p-3">
                <div class="mb-2 flex items-center space-x-2">
                  <div class="h-2 w-2 rounded-full bg-green-500"></div>
                  <p class="text-sm font-medium text-foreground">Microphone Connected</p>
                </div>
                <Select.Root type="single" bind:value={selectedMicrophoneId}>
                  <Select.Trigger class="w-full">
                    {availableMicrophones.find((m) => m.deviceId === selectedMicrophoneId)?.label ||
                      `Microphone ${selectedMicrophoneId.slice(0, 8)}` ||
                      'Select a microphone'}
                  </Select.Trigger>
                  <Select.Content>
                    {#each availableMicrophones as microphone}
                      <Select.Item value={microphone.deviceId}>
                        {microphone.label || `Microphone ${microphone.deviceId.slice(0, 8)}`}
                      </Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <Separator />

      <!-- Recording Section -->
      <div class="space-y-4">
        <!-- Status Display -->
        {#if recordingError}
          <p class="text-sm text-destructive">{recordingError}</p>
        {/if}

        {#if recordingState === 'not-ready'}
          <div class="space-y-4">
            <Button onclick={handleRecord} class="w-full md:w-auto">
              <Play class="mr-2 h-4 w-4" />
              Start Recording
            </Button>
          </div>
        {:else if recordingState === 'ready'}
          <div class="space-y-2">
            <Button onclick={handleRecord} class="w-full md:w-auto">
              <Play class="mr-2 h-4 w-4" />
              Start Recording
            </Button>
          </div>
        {:else if isRecording || isPaused}
          <div class="space-y-2">
            <div class="space-y-3">
              <div
                class="relative w-full rounded-lg border p-4 {isRecording
                  ? 'border-red-200 bg-red-50 text-red-700'
                  : 'border-yellow-200 bg-yellow-50 text-yellow-700'}"
              >
                <div class="flex items-center space-x-2">
                  <div class="h-4 w-4 rounded-full {isRecording ? 'animate-pulse bg-red-500' : 'bg-yellow-500'}"></div>
                  <p class="text-sm font-medium {isRecording ? 'text-red-800' : 'text-yellow-800'}">
                    {isRecording ? 'Recording in progress...' : 'Recording paused...'}
                  </p>
                </div>
                <div class="mt-2 space-y-1 pl-6">
                  <p class="text-xs {isRecording ? 'text-red-700' : 'text-yellow-700'}">
                    Patient: {formData.firstName}
                    {formData.lastName}
                  </p>
                  <p class="text-xs {isRecording ? 'text-red-700' : 'text-yellow-700'}">
                    Note Type: {formData.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
                  </p>
                  <p class="text-sm font-medium {isRecording ? 'text-red-800' : 'text-yellow-800'}">
                    Duration: {formatTime(recordingTime)}
                  </p>
                </div>
              </div>
              <div class="flex gap-2">
                <Button onclick={handlePauseResume} variant="outline" disabled={!canPauseResume}>
                  {#if isPaused}
                    <Play class="mr-2 h-4 w-4" />
                  {:else}
                    <Pause class="mr-2 h-4 w-4" />
                  {/if}
                  {isPaused ? 'Resume' : 'Pause'}
                </Button>
                <Button onclick={handleStopRecording} variant="destructive">
                  <Square class="mr-2 h-4 w-4" />
                  Stop Recording
                </Button>
              </div>
            </div>
          </div>
        {:else if recordingState === 'stopped'}
          <div class="space-y-2">
            <div class="space-y-3">
              <div class="relative w-full rounded-lg border border-green-500/50 bg-green-50 p-4 text-green-700">
                <CheckCircle class="absolute top-4 left-4 h-4 w-4" />
                <div class="pl-7">
                  <h5 class="mb-1 leading-none font-medium tracking-tight">Recording completed.</h5>
                  <div class="mt-2 space-y-1">
                    <p class="text-xs text-green-600">
                      Patient: {formData.firstName}
                      {formData.lastName}
                    </p>
                    <p class="text-xs text-green-600">
                      Note Type: {formData.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
                    </p>
                    <p class="text-xs text-green-600">
                      Recording Duration: {formatTime(recordingTime)}
                    </p>
                  </div>
                </div>
              </div>

              <!-- Processing Errors -->
              {#if processingError}
                <p class="text-sm text-destructive">{processingError}</p>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </CardContent>
  </Card>
</div>
