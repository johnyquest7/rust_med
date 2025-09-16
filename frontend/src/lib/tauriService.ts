import { browser } from '$app/environment';
import type { TauriNote, TauriNoteIn } from '$lib/types';

declare global {
  interface Window {
    __TAURI__: {
      core: { invoke: (command: string, args?: any) => Promise<any> };
      fs: { writeFile: (path: string, data: string | Uint8Array) => Promise<void> };
      path: { appLocalDataDir: () => Promise<string> };
      event: { listen: (event: string, callback: (data: any) => void) => Promise<void> };
    };
  }
}

class TauriService {
  private tauri: typeof window.__TAURI__ | null = null;

  private ensureTauri(): typeof window.__TAURI__ {
    if (!this.tauri) {
      // Don't initialize during SSR, only in browser
      if (!browser) {
        throw new Error('Tauri APIs not available outside of browser');
      }
      if (typeof window.__TAURI__ === 'undefined') {
        throw new Error('Tauri APIs not available');
      }
      this.tauri = window.__TAURI__;
    }
    return this.tauri;
  }

  async appLocalDataDir(): Promise<string> {
    return this.ensureTauri().path.appLocalDataDir();
  }

  async joinPath(directory: string, filename: string): Promise<string> {
    // @ts-ignore
    return await this.ensureTauri().path.join(directory, filename);
  }

  async makePath(directory: string, filename: string): Promise<string> {
    // Use Tauri's path.join to construct the path in a cross-platform way
    // @ts-ignore
    return await this.ensureTauri().path.join(directory, filename);
  }

  async writeFile(path: string, data: string | Uint8Array): Promise<void> {
    await this.ensureTauri().fs.writeFile(path, data);
  }

  async transcribeAudio(audioPath: string): Promise<{ success: boolean; transcript: string; error: string | null }> {
    return this.ensureTauri().core.invoke('transcribe_audio', {
      audioPath: audioPath
    });
  }

  async generateMedicalNote(
    transcript: string,
    noteType: string
  ): Promise<{ success: boolean; note: string; error: string | null }> {
    const result = await this.ensureTauri().core.invoke('generate_medical_note', {
      transcript: transcript,
      noteType: noteType
    });
    return result;
  }

  async loadNotes(): Promise<{ success: boolean; notes: TauriNote[]; error: string | null }> {
    const result = await this.ensureTauri().core.invoke('load_patient_notes');
    if (result.success) {
      const notes = result.notes.map((n: any) => ({
        id: n.id,
        firstName: n.first_name,
        lastName: n.last_name,
        dateOfBirth: n.date_of_birth,
        noteType: n.note_type,
        transcript: n.transcript,
        medicalNote: n.medical_note,
        createdAt: n.created_at
      }));
      return { success: true, notes: notes, error: null };
    }
    return { success: false, notes: [], error: result.error };
  }

  async createNote(note: TauriNoteIn): Promise<{ success: boolean; note_id: string | null; error: string | null }> {
    return await this.ensureTauri().core.invoke('create_patient_note', note);
  }

  async deleteNote(noteId: string): Promise<{ success: boolean; error: string | null }> {
    return await this.ensureTauri().core.invoke('delete_patient_note', { noteId: noteId });
  }

  async updateNote(
    noteId: string,
    note: TauriNoteIn
  ): Promise<{ success: boolean; note_id: string | null; error: string | null }> {
    return await this.ensureTauri().core.invoke('update_patient_note', {
      noteId: noteId,
      ...note
    });
  }
}

// Create a singleton instance
export const tauriService = new TauriService();
