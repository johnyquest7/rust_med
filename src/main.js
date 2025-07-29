// Medical Note Generator - Main Entry Point
import { UIManager } from './modules/uiManager.js';
import { RecordingManager } from './modules/recordingManager.js';
import { TranscriptionManager } from './modules/transcriptionManager.js';
import { NotesManager } from './modules/notesManager.js';
import { hasTauriAPIs, handleError } from './modules/utils.js';

console.log('Loading Medical Note Generator...');

class MedicalNoteGenerator {
    constructor() {
        this.ui = new UIManager();
        this.recording = new RecordingManager();
        this.transcription = new TranscriptionManager();
        this.notes = new NotesManager();
        
        this.lastTranscript = '';
        this.lastMedicalNote = '';
        
        this.initialize();
    }

    async initialize() {
        try {
            // Load microphones
            await this.ui.loadMicrophones();
            
            // Load saved notes
            await this.notes.loadNotes();
            
            // Bind event handlers
            this.bindEvents();
            
            // Setup transcription event listeners
            this.setupTranscriptionListeners();
            
            // Setup notes event listeners
            this.setupNotesListeners();
            
            // Set initial status
            this.ui.updateStatus('Ready');
            
            console.log('Medical Note Generator initialized successfully');
        } catch (error) {
            console.error('Failed to initialize app:', error);
            this.ui.showError('Failed to initialize application');
        }
    }

    setupTranscriptionListeners() {
        // Listen for transcription progress updates
        this.transcription.onTranscriptionProgress((message, isText = false) => {
            if (isText) {
                // This is the actual transcription text
                this.ui.showTranscript(message);
                this.lastTranscript = message;
            } else {
                // This is a progress message
                this.ui.showTranscript(message);
            }
        });

        // Listen for note generation progress
        this.transcription.onNoteGenerationProgress((message) => {
            this.ui.showMedicalNote(message);
            if (!message.includes('Generating')) {
                this.lastMedicalNote = message;
            }
        });
        
        // Listen for note generation streaming
        this.transcription.onNoteGenerationStream((streamedNote) => {
            this.ui.showMedicalNote(streamedNote);
        });
        
        // Listen for note generation completion
        this.transcription.onNoteGenerationComplete((finalNote) => {
            this.ui.showMedicalNote(finalNote);
            this.lastMedicalNote = finalNote;
            this.ui.updateStatus('Medical note generated successfully!');
            this.ui.enableSaveButton(true);
        });
    }

    setupNotesListeners() {
        // Listen for notes updates
        this.notes.onNotesUpdate((notes) => {
            const groupedNotes = this.notes.groupNotesByDate();
            this.ui.updateNotesList(groupedNotes);
        });

        // Listen for note selection
        this.notes.onNoteSelected((note) => {
            if (note) {
                this.ui.displaySelectedNote(note);
                this.ui.enableSaveButton(false); // Disable save when viewing existing note
            }
        });

        // Setup note click handler
        this.ui.bindNoteClickHandler((noteId) => {
            this.notes.selectNote(noteId);
        });
    }

    bindEvents() {
        // Recording controls
        this.ui.bindStartButton(() => this.handleStartRecording());
        this.ui.bindStopButton(() => this.handleStopRecording());
        this.ui.bindCopyButton(() => this.ui.copyNote());
        this.ui.bindTestButton(() => this.ui.testMicrophone());
        this.ui.bindSaveButton(() => this.handleSaveNote());
    }

    async handleStartRecording() {
        try {
            // Validate patient info first
            if (!this.ui.validatePatientInfo()) {
                return;
            }
            
            this.ui.updateStatus('Initializing recording...');
            this.ui.clearResults();
            this.ui.enableSaveButton(false);
            
            // Clear last results
            this.lastTranscript = '';
            this.lastMedicalNote = '';
            
            const deviceId = this.ui.getSelectedDeviceId();
            await this.recording.startRecording(deviceId);
            
            this.ui.startTimer();
            this.ui.updateRecordingState(true);
            
        } catch (error) {
            const errorMessage = handleError(error, 'start recording');
            this.ui.showError(errorMessage);
            this.ui.updateRecordingState(false);
        }
    }

    async handleStopRecording() {
        try {
            this.ui.updateStatus('Stopping recording...');
            
            this.recording.stopRecording();
            this.ui.stopTimer();
            this.ui.updateRecordingState(false);
            
            // Process the recorded audio
            await this.processRecording();
            
        } catch (error) {
            const errorMessage = handleError(error, 'stop recording');
            this.ui.showError(errorMessage);
            this.ui.updateRecordingState(false);
        }
    }

    async processRecording() {
        try {
            this.ui.updateStatus('Converting audio to WAV format...');
            
            // Process and convert audio
            const convertedBlob = await this.recording.processRecordedAudio();
            
            if (hasTauriAPIs) {
                await this.handleTauriTranscription(convertedBlob);
            } else {
                this.handleStandaloneMode(convertedBlob);
            }
            
        } catch (error) {
            const errorMessage = handleError(error, 'process recording');
            this.ui.showError(errorMessage);
        }
    }

    async handleTauriTranscription(convertedBlob) {
        try {
            this.ui.updateStatus('Transcribing audio...');
            
            const noteType = this.ui.getSelectedNoteType();
            const result = await this.transcription.saveAndTranscribe(convertedBlob, noteType);
            
            if (result.success) {
                // The transcript is already shown via events, just store it
                this.lastTranscript = result.transcript;
                
                if (result.medicalNote) {
                    // Note generation succeeded - the note is already shown via events
                    this.ui.showMedicalNote(result.medicalNote);
                    this.lastMedicalNote = result.medicalNote;
                    this.ui.updateStatus('Medical note generated successfully!');
                    this.ui.enableSaveButton(true); // Enable save button
                } else if (result.noteError) {
                    // Transcription succeeded but note generation failed
                    this.ui.showMedicalNote(`Error generating note: ${result.noteError}`);
                    this.ui.updateStatus('Transcription completed (note generation failed)');
                    this.ui.enableSaveButton(false);
                }
            }
            
        } catch (error) {
            const errorMessage = handleError(error, 'transcription');
            this.ui.showError(errorMessage);
            this.ui.showTranscript(`Transcription failed: ${errorMessage}`);
            this.ui.showMedicalNote('Audio saved successfully, but transcription service failed.');
        }
    }

    handleStandaloneMode(convertedBlob) {
        const timerText = this.ui.getTimerText();
        const selectedMicName = this.ui.getSelectedMicrophoneName();
        
        this.ui.showStandaloneResults(convertedBlob, timerText, selectedMicName);
        this.ui.updateStatus('Recording completed successfully!');
    }

    async handleSaveNote() {
        try {
            // Validate patient info
            if (!this.ui.validatePatientInfo()) {
                return;
            }

            // Check if we have both transcript and medical note
            if (!this.lastTranscript || !this.lastMedicalNote) {
                this.ui.showError('No note to save. Please record and generate a note first.');
                return;
            }

            this.ui.updateStatus('Saving note...');
            this.ui.enableSaveButton(false);

            const patientInfo = this.ui.getPatientInfo();
            const noteType = this.ui.getSelectedNoteType();

            const result = await this.notes.saveNote(
                patientInfo,
                noteType,
                this.lastTranscript,
                this.lastMedicalNote
            );

            if (result.success) {
                this.ui.updateStatus('Note saved successfully!');
                this.ui.clearResults();
                this.ui.clearPatientInfo();
                this.lastTranscript = '';
                this.lastMedicalNote = '';
                
                // The notes list will be automatically updated via the listener
            } else {
                throw new Error(result.error || 'Failed to save note');
            }

        } catch (error) {
            const errorMessage = handleError(error, 'save note');
            this.ui.showError(errorMessage);
            this.ui.enableSaveButton(true); // Re-enable save button on error
        }
    }

    // Public methods for external access
    reset() {
        this.recording.reset();
        this.transcription.clearResults();
        this.ui.clearResults();
        this.ui.updateStatus('Ready');
        this.lastTranscript = '';
        this.lastMedicalNote = '';
        this.ui.enableSaveButton(false);
    }

    getLastResults() {
        return {
            transcript: this.transcription.getLastTranscript(),
            medicalNote: this.transcription.getLastMedicalNote()
        };
    }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    try {
        const app = new MedicalNoteGenerator();
        
        // Make app available globally for debugging
        window.medicalNoteApp = app;
        
    } catch (error) {
        console.error('Failed to initialize app:', error);
    }
});