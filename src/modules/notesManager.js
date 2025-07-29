// Notes Manager Module - Handles saving, loading, and managing patient notes
import { hasTauriAPIs, invoke } from './utils.js';

export class NotesManager {
    constructor() {
        this.notes = [];
        this.selectedNoteId = null;
        this.onNotesUpdateListeners = [];
        this.onNoteSelectedListeners = [];
    }

    onNotesUpdate(callback) {
        this.onNotesUpdateListeners.push(callback);
    }

    onNoteSelected(callback) {
        this.onNoteSelectedListeners.push(callback);
    }

    notifyNotesUpdate() {
        this.onNotesUpdateListeners.forEach(callback => callback(this.notes));
    }

    notifyNoteSelected(note) {
        this.onNoteSelectedListeners.forEach(callback => callback(note));
    }

    async saveNote(patientInfo, noteType, transcript, medicalNote) {
        if (!hasTauriAPIs) {
            throw new Error('Tauri APIs not available');
        }

        try {
            const result = await invoke('save_patient_note', {
                firstName: patientInfo.firstName,
                lastName: patientInfo.lastName,
                dob: patientInfo.dob,
                noteType: noteType,
                transcript: transcript,
                medicalNote: medicalNote
            });

            if (result.success) {
                // Reload notes after saving
                await this.loadNotes();
                return result;
            } else {
                throw new Error(result.error || 'Failed to save note');
            }
        } catch (error) {
            console.error('Failed to save note:', error);
            throw error;
        }
    }

    async loadNotes() {
        if (!hasTauriAPIs) {
            console.warn('Tauri APIs not available - using mock data');
            this.notes = [];
            this.notifyNotesUpdate();
            return;
        }

        try {
            const result = await invoke('load_patient_notes');
            
            if (result.success) {
                this.notes = result.notes;
                this.notifyNotesUpdate();
            } else {
                throw new Error(result.error || 'Failed to load notes');
            }
        } catch (error) {
            console.error('Failed to load notes:', error);
            this.notes = [];
            this.notifyNotesUpdate();
        }
    }

    async deleteNote(noteId) {
        if (!hasTauriAPIs) {
            throw new Error('Tauri APIs not available');
        }

        try {
            const success = await invoke('delete_patient_note', { noteId: noteId });
            
            if (success) {
                // Clear selection if deleted note was selected
                if (this.selectedNoteId === noteId) {
                    this.selectedNoteId = null;
                    this.notifyNoteSelected(null);
                }
                
                // Reload notes after deletion
                await this.loadNotes();
                return true;
            } else {
                throw new Error('Failed to delete note');
            }
        } catch (error) {
            console.error('Failed to delete note:', error);
            throw error;
        }
    }

    selectNote(noteId) {
        const note = this.notes.find(n => n.id === noteId);
        if (note) {
            this.selectedNoteId = noteId;
            this.notifyNoteSelected(note);
        }
    }

    getSelectedNote() {
        return this.notes.find(n => n.id === this.selectedNoteId) || null;
    }

    groupNotesByDate() {
        const grouped = {};
        const today = new Date().toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric'
        });
        
        this.notes.forEach(note => {
            // Parse the created_at timestamp
            const date = new Date(note.created_at);
            const dateKey = date.toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric'
            });
            
            if (!grouped[dateKey]) {
                grouped[dateKey] = [];
            }
            
            grouped[dateKey].push(note);
        });
        
        // Sort dates in descending order
        const sortedDates = Object.keys(grouped).sort((a, b) => {
            return new Date(b) - new Date(a);
        });
        
        const result = [];
        sortedDates.forEach(date => {
            result.push({
                date: date,
                notes: grouped[date],
                isToday: date === today
            });
        });
        
        return result;
    }
    
    getTodaysNotes() {
        const today = new Date();
        today.setHours(0, 0, 0, 0); // Start of today
        const tomorrow = new Date(today);
        tomorrow.setDate(tomorrow.getDate() + 1); // Start of tomorrow
        
        return this.notes.filter(note => {
            const noteDate = new Date(note.created_at);
            return noteDate >= today && noteDate < tomorrow;
        });
    }
    
    getTodaysNotesCount() {
        return this.getTodaysNotes().length;
    }

    formatNoteDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }
}