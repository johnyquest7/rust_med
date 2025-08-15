// Recording Manager Module - Handles audio recording functionality
import { AudioConverter } from './audioConverter.js';

export class RecordingManager {
    constructor() {
        this.isRecording = false;
        this.isPaused = false;
        this.mediaRecorder = null;
        this.audioChunks = [];
        this.recordedBlob = null;
        this.convertedBlob = null;
        this.selectedFormat = null;
        this.stream = null;
        this.checkPauseResumeSupport();
    }

    checkPauseResumeSupport() {
        // Check if the browser supports pause/resume
        if (typeof MediaRecorder !== 'undefined' && MediaRecorder.prototype.pause && MediaRecorder.prototype.resume) {
            console.log('Browser supports MediaRecorder pause/resume');
            this.pauseResumeSupported = true;
        } else {
            console.warn('Browser does not support MediaRecorder pause/resume');
            this.pauseResumeSupported = false;
        }
    }

    async startRecording(deviceId) {
        try {
            // Enhanced audio constraints for better speech recognition
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

            this.stream = await navigator.mediaDevices.getUserMedia({
                audio: audioConstraints
            });

            // Prefer WebM format for better quality, will convert to WAV
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

            this.selectedFormat = null;
            for (const mimeType of supportedFormats) {
                if (MediaRecorder.isTypeSupported(mimeType)) {
                    this.selectedFormat = { mime: mimeType, needsConversion: true };
                    break;
                }
            }

            if (!this.selectedFormat) {
                throw new Error('No audio recording formats supported by your browser');
            }

            this.mediaRecorder = new MediaRecorder(this.stream, {
                mimeType: this.selectedFormat.mime,
                audioBitsPerSecond: 128000
            });

            this.audioChunks = [];
            this.isRecording = true;

            return new Promise((resolve, reject) => {
                this.mediaRecorder.ondataavailable = (event) => {
                    if (event.data.size > 0) {
                        this.audioChunks.push(event.data);
                    }
                };

                this.mediaRecorder.onstop = () => {
                    if (!this.isPaused) {
                        // Normal stop
                        this.isRecording = false;
                        resolve('Recording stopped successfully');
                    } else {
                        // Paused stop - keep recording state true
                        console.log('Recording paused (via stop)');
                    }
                };

                this.mediaRecorder.onerror = (event) => {
                    this.isRecording = false;
                    this.cleanup();
                    reject(new Error('Recording error: ' + event.error.message));
                };

                this.mediaRecorder.onstart = () => {
                    console.log('MediaRecorder started');
                    resolve('Recording started successfully');
                };

                this.mediaRecorder.onpause = () => {
                    console.log('MediaRecorder paused');
                };

                this.mediaRecorder.onresume = () => {
                    console.log('MediaRecorder resumed');
                };

                this.mediaRecorder.start(500); // Record in 500ms chunks
            });

        } catch (error) {
            this.isRecording = false;
            this.cleanup();
            throw new Error(`Failed to start recording: ${error.message}`);
        }
    }

    pauseRecording() {
        try {
            console.log('Attempting to pause recording. MediaRecorder state:', this.mediaRecorder?.state);
            console.log('Pause/resume supported:', this.pauseResumeSupported);

            if (this.mediaRecorder && this.mediaRecorder.state === 'recording') {
                if (this.pauseResumeSupported) {
                    // Use native pause if supported
                    this.mediaRecorder.pause();
                } else {
                    // Fallback: stop recording but keep the stream
                    this.mediaRecorder.stop();
                    // The onstop handler will be triggered, but we'll handle it differently when paused
                }
                this.isPaused = true;
                console.log('Recording paused successfully');
                return true;
            } else {
                throw new Error(`Recording not active. Current state: ${this.mediaRecorder?.state || 'no recorder'}`);
            }
        } catch (error) {
            console.error('Error in pauseRecording:', error);
            throw new Error(`Error pausing recording: ${error.message}`);
        }
    }

    async resumeRecording() {
        try {
            console.log('Attempting to resume recording. MediaRecorder state:', this.mediaRecorder?.state);

            if (this.pauseResumeSupported && this.mediaRecorder && this.mediaRecorder.state === 'paused') {
                // Use native resume if supported
                this.mediaRecorder.resume();
                this.isPaused = false;
                console.log('Recording resumed successfully');
                return true;
            } else if (!this.pauseResumeSupported && this.isPaused && this.stream) {
                // Fallback: create new MediaRecorder with same stream
                this.mediaRecorder = new MediaRecorder(this.stream, {
                    mimeType: this.selectedFormat.mime,
                    audioBitsPerSecond: 128000
                });

                // Re-attach event handlers
                this.mediaRecorder.ondataavailable = (event) => {
                    if (event.data.size > 0) {
                        this.audioChunks.push(event.data);
                    }
                };

                this.mediaRecorder.onstop = () => {
                    if (!this.isPaused) {
                        this.isRecording = false;
                    }
                };

                this.mediaRecorder.onstart = () => {
                    console.log('MediaRecorder resumed (restarted)');
                };

                this.mediaRecorder.start(500);
                this.isPaused = false;
                console.log('Recording resumed successfully (via restart)');
                return true;
            } else {
                throw new Error(`Cannot resume. isPaused: ${this.isPaused}, stream exists: ${!!this.stream}`);
            }
        } catch (error) {
            console.error('Error in resumeRecording:', error);
            throw new Error(`Error resuming recording: ${error.message}`);
        }
    }

    stopRecording() {
        try {
            if (this.mediaRecorder && (this.mediaRecorder.state === 'recording' || this.mediaRecorder.state === 'paused')) {
                this.mediaRecorder.stop();
                this.isPaused = false;
                return true;
            } else {
                throw new Error('Recording not active');
            }
        } catch (error) {
            this.cleanup();
            throw new Error(`Error stopping recording: ${error.message}`);
        }
    }

    async processRecordedAudio() {
        try {
            if (this.audioChunks.length === 0) {
                throw new Error('No audio data recorded');
            }

            this.recordedBlob = new Blob(this.audioChunks, { type: this.selectedFormat.mime });

            // Convert to WAV for better whisperfile compatibility
            try {
                this.convertedBlob = await AudioConverter.webmToWav(this.recordedBlob);
                return this.convertedBlob;
            } catch (conversionError) {
                throw new Error('Audio conversion failed: ' + conversionError.message);
            }

        } catch (error) {
            this.cleanup();
            throw new Error(`Failed to process audio: ${error.message}`);
        }
    }

    cleanup() {
        if (this.stream) {
            this.stream.getTracks().forEach(track => track.stop());
            this.stream = null;
        }
        if (this.mediaRecorder) {
            this.mediaRecorder = null;
        }
        this.isRecording = false;
        this.isPaused = false;
    }

    getRecordedBlob() {
        return this.recordedBlob;
    }

    getConvertedBlob() {
        return this.convertedBlob;
    }

    getIsRecording() {
        return this.isRecording;
    }

    getIsPaused() {
        return this.isPaused;
    }

    getStream() {
        return this.stream;
    }

    reset() {
        this.cleanup();
        this.audioChunks = [];
        this.recordedBlob = null;
        this.convertedBlob = null;
        this.selectedFormat = null;
        this.isPaused = false;
    }
}