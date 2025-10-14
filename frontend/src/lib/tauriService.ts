import { browser } from '$app/environment';
import type { TauriNote, TauriNoteIn, AuthResponse, CreateUserRequest, AuthenticateRequest, ModelInfo, ModelPreferences, DownloadedModel, WhisperModelMetadata, RuntimeBinaryMetadata, MedLlamaModelMetadata } from '$lib/types';
import { authContext } from '$lib/hooks/auth-context.svelte.js';

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
    const password = authContext.getPassword();
    if (!password) {
      return { success: false, notes: [], error: 'Password required for decryption' };
    }

    const result = await this.ensureTauri().core.invoke('load_patient_notes', { password });
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
    const password = authContext.getPassword();
    if (!password) {
      console.error('No password available for encryption');
      return { success: false, note_id: null, error: 'Password required for encryption' };
    }

    console.log('Creating note with password available');
    const result = await this.ensureTauri().core.invoke('create_patient_note', {
      password,
      firstName: note.firstName,
      lastName: note.lastName,
      dateOfBirth: note.dateOfBirth,
      noteType: note.noteType,
      transcript: note.transcript,
      medicalNote: note.medicalNote
    });
    
    console.log('Create note result:', result);
    return result;
  }

  async deleteNote(noteId: string): Promise<boolean> {
    return await this.ensureTauri().core.invoke('delete_patient_note', { noteId: noteId });
  }

  async updateNote(
    noteId: string,
    note: TauriNoteIn
  ): Promise<{ success: boolean; note_id: string | null; error: string | null }> {
    const password = authContext.getPassword();
    if (!password) {
      return { success: false, note_id: null, error: 'Password required for encryption' };
    }

    return await this.ensureTauri().core.invoke('update_patient_note', {
      password,
      noteId: noteId,
      firstName: note.firstName,
      lastName: note.lastName,
      dateOfBirth: note.dateOfBirth,
      noteType: note.noteType,
      transcript: note.transcript,
      medicalNote: note.medicalNote
    });
  }

  // Authentication methods
  async authenticateUser(request: AuthenticateRequest): Promise<AuthResponse> {
    return await this.ensureTauri().core.invoke('authenticate_user_command', { request });
  }

  async createUserAccount(request: CreateUserRequest): Promise<AuthResponse> {
    return await this.ensureTauri().core.invoke('create_user_account_command', { request });
  }

  async checkAuthStatus(): Promise<AuthResponse> {
    return await this.ensureTauri().core.invoke('check_auth_status');
  }

  async deleteAudioFile(audioPath: string): Promise<boolean> {
    return await this.ensureTauri().core.invoke('delete_audio_file', { audioPath });
  }

  // Setup wizard methods
  async getRequiredModelsList(): Promise<any[]> {
    return await this.ensureTauri().core.invoke('get_required_models_list');
  }

  async checkModelsDownloaded(): Promise<[any, boolean][]> {
    return await this.ensureTauri().core.invoke('check_models_downloaded');
  }

  async checkAllModelsInstalled(): Promise<boolean> {
    return await this.ensureTauri().core.invoke('check_all_models_installed');
  }

  async getModelsInfo(): Promise<ModelInfo[]> {
    return await this.ensureTauri().core.invoke('get_models_info_command');
  }

  async downloadModelFile(model: any): Promise<void> {
    return await this.ensureTauri().core.invoke('download_model_file', { model });
  }

  async completeSetup(): Promise<void> {
    return await this.ensureTauri().core.invoke('complete_setup');
  }

  async listen<T>(event: string, callback: (data: { payload: T }) => void): Promise<void> {
    return await this.ensureTauri().event.listen(event, callback);
  }

  // Model preference methods
  async getModelPreferences(): Promise<ModelPreferences> {
    return await this.ensureTauri().core.invoke('get_model_preferences_command');
  }

  async saveModelPreferences(preferences: ModelPreferences): Promise<boolean> {
    return await this.ensureTauri().core.invoke('save_model_preferences_command', { preferences });
  }

  async listDownloadedModels(): Promise<DownloadedModel[]> {
    return await this.ensureTauri().core.invoke('list_downloaded_models');
  }

  async deleteModelFile(filename: string): Promise<boolean> {
    return await this.ensureTauri().core.invoke('delete_model_file', { filename });
  }

  async downloadCustomModel(url: string, filename: string): Promise<string> {
    return await this.ensureTauri().core.invoke('download_custom_model', { url, filename });
  }

  // Model metadata methods (SINGLE SOURCE OF TRUTH from backend)
  async getWhisperModelOptions(): Promise<WhisperModelMetadata[]> {
    return await this.ensureTauri().core.invoke('get_whisper_model_options_command');
  }

  async getRuntimeBinaries(): Promise<RuntimeBinaryMetadata[]> {
    return await this.ensureTauri().core.invoke('get_runtime_binaries_command');
  }

  async getMedLlamaMetadata(): Promise<MedLlamaModelMetadata> {
    return await this.ensureTauri().core.invoke('get_medllama_metadata_command');
  }
}

// Create a singleton instance
export const tauriService = new TauriService();
