// Transcription Manager Module - Handles audio transcription and medical note generation
import { hasTauriAPIs, invoke, writeFile, listen } from './utils.js';

export class TranscriptionManager {
    constructor() {
        this.lastTranscript = '';
        this.lastMedicalNote = '';
        this.transcriptionListeners = [];
        this.noteGenerationListeners = [];
        this.setupEventListeners();
    }

    setupEventListeners() {
        if (!hasTauriAPIs) return;

        // Listen for transcription progress events
        listen('transcription-progress', (event) => {
            console.log('Transcription progress:', event.payload);
            this.notifyTranscriptionProgress(event.payload);
        });

        // Listen for transcription text events
        listen('transcription-text', (event) => {
            console.log('Transcription text:', event.payload);
            this.notifyTranscriptionText(event.payload);
        });

        // Listen for note generation progress events
        listen('note-generation-progress', (event) => {
            console.log('Note generation progress:', event.payload);
            this.notifyNoteGenerationProgress(event.payload);
        });
    }

    onTranscriptionProgress(callback) {
        this.transcriptionListeners.push(callback);
    }

    onNoteGenerationProgress(callback) {
        this.noteGenerationListeners.push(callback);
    }

    notifyTranscriptionProgress(message) {
        this.transcriptionListeners.forEach(callback => callback(message));
    }

    notifyTranscriptionText(text) {
        this.transcriptionListeners.forEach(callback => callback(text, true));
    }

    notifyNoteGenerationProgress(message) {
        this.noteGenerationListeners.forEach(callback => callback(message));
    }

    async saveAndTranscribe(convertedBlob, noteType = 'soap') {
        if (!hasTauriAPIs) {
            throw new Error('Tauri APIs not available');
        }

        try {
            // Ensure app directory exists
            const appDataDir = await invoke('ensure_app_directory');
            
            // Convert blob to array buffer and save
            const audioBuffer = await convertedBlob.arrayBuffer();
            const uint8Array = new Uint8Array(audioBuffer);
            const audioPath = `${appDataDir}\\recording.wav`;
            
            await writeFile(audioPath, uint8Array);

            // Validate the saved file
            await invoke('validate_audio_file', { audioPath: audioPath });

            // Transcribe the audio
            const transcriptionResult = await invoke('transcribe_audio', { audioPath: audioPath });
            
            if (!transcriptionResult.success) {
                throw new Error(`Transcription failed: ${transcriptionResult.error}`);
            }

            this.lastTranscript = transcriptionResult.transcript;

            // Generate medical note
            const noteResult = await invoke('generate_medical_note', {
                transcript: transcriptionResult.transcript,
                noteType: noteType
            });
            
            if (noteResult.success) {
                this.lastMedicalNote = noteResult.note;
                return {
                    success: true,
                    transcript: this.lastTranscript,
                    medicalNote: this.lastMedicalNote
                };
            } else {
                // Transcription succeeded but note generation failed
                return {
                    success: true,
                    transcript: this.lastTranscript,
                    medicalNote: null,
                    noteError: noteResult.error
                };
            }
            
        } catch (error) {
            throw new Error(`Transcription process failed: ${error.message}`);
        }
    }

    getLastTranscript() {
        return this.lastTranscript;
    }

    getLastMedicalNote() {
        return this.lastMedicalNote;
    }

    clearResults() {
        this.lastTranscript = '';
        this.lastMedicalNote = '';
    }
}