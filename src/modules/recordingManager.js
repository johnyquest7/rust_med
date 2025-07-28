// Recording Manager Module - Handles audio recording functionality
import { AudioConverter } from './audioConverter.js';

export class RecordingManager {
    constructor() {
        this.isRecording = false;
        this.mediaRecorder = null;
        this.audioChunks = [];
        this.recordedBlob = null;
        this.convertedBlob = null;
        this.selectedFormat = null;
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

            const stream = await navigator.mediaDevices.getUserMedia({ 
                audio: audioConstraints
            });

            // Prefer WebM format for better quality, will convert to WAV
            const supportedFormats = [
                'audio/webm;codecs=opus',
                'audio/webm', 
                'audio/ogg;codecs=opus',
                'audio/ogg'
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

            this.mediaRecorder = new MediaRecorder(stream, {
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
                    this.isRecording = false;
                    resolve('Recording stopped successfully');
                };

                this.mediaRecorder.onerror = (event) => {
                    this.isRecording = false;
                    this.cleanup();
                    reject(new Error('Recording error: ' + event.error.message));
                };

                this.mediaRecorder.onstart = () => {
                    resolve('Recording started successfully');
                };

                this.mediaRecorder.start(500); // Record in 500ms chunks
            });

        } catch (error) {
            this.isRecording = false;
            this.cleanup();
            throw new Error(`Failed to start recording: ${error.message}`);
        }
    }

    stopRecording() {
        try {
            if (this.mediaRecorder && this.mediaRecorder.state === 'recording') {
                this.mediaRecorder.stop();
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
        if (this.mediaRecorder) {
            if (this.mediaRecorder.stream) {
                this.mediaRecorder.stream.getTracks().forEach(track => track.stop());
            }
            this.mediaRecorder = null;
        }
        this.isRecording = false;
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

    reset() {
        this.cleanup();
        this.audioChunks = [];
        this.recordedBlob = null;
        this.convertedBlob = null;
        this.selectedFormat = null;
    }
}