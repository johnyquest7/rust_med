/**
 * Type definitions for the Medical Note Generator application
 */

/**
 * User information interface (simplified for authentication)
 */
export interface User {
  /** Unique user identifier */
  user_id: string;
  /** Username for login */
  username: string;
}

/**
 * Authentication state interface
 */
export interface AuthState {
  /** Current user if authenticated, null if not */
  user: User | null;
  /** Whether the user is currently authenticated */
  isAuthenticated: boolean;
  /** Whether authentication is currently being processed */
  isLoading: boolean;
  /** Any authentication error message */
  error: string | null;
}

/**
 * Login credentials interface
 */
export interface LoginCredentials {
  /** Username for login */
  username: string;
  /** Password for login */
  password: string;
}

/**
 * User registration data interface (simplified)
 */
export interface RegisterData {
  /** Username for login */
  username: string;
  /** Password for login */
  password: string;
}

/**
 * Authentication response from backend
 */
export interface AuthResponse {
  /** Whether the operation was successful */
  success: boolean;
  /** Response message */
  message: string;
  /** User information if successful */
  user: User | null;
}

/**
 * Create user account request
 */
export interface CreateUserRequest {
  /** Username for login */
  username: string;
  /** Password for login */
  password: string;
}

/**
 * Authenticate user request
 */
export interface AuthenticateRequest {
  /** Password for login */
  password: string;
}

/**
 * Authentication context methods interface
 */
export interface AuthContext {
  /** Current authentication state */
  state: AuthState;
  /** Login with password (username shown from auth file) */
  login: (password: string) => Promise<void>;
  /** Register new user account */
  register: (data: RegisterData) => Promise<void>;
  /** Logout the current user */
  logout: () => void;
  /** Clear any authentication errors */
  clearError: () => void;
  /** Check if user is authenticated */
  isAuthenticated: () => boolean;
  /** Check authentication status on app startup */
  checkAuthStatus: () => Promise<void>;
  /** Initialize authentication state from localStorage */
  initialize: () => void;
  /** Get the current password for encryption (only available when authenticated) */
  getPassword: () => string | null;
}

export interface TauriNoteIn {
  firstName: string;
  lastName: string;
  dateOfBirth: string;
  noteType: string;
  transcript: string;
  medicalNote: string;
}

export interface TauriNote extends TauriNoteIn {
  id: string;
  createdAt: string;
}

export type RecordingState = 'not-ready' | 'ready' | 'recording' | 'paused' | 'stopped' | 'error';

/**
 * Model information interface
 */
export interface ModelInfo {
  /** Display name of the model */
  name: string;
  /** File name of the model */
  file_name: string;
  /** Size in megabytes */
  size_mb: number;
  /** Whether the model is installed */
  installed: boolean;
  /** File path if installed */
  file_path: string | null;
}

/**
 * Whisper model size options
 */
export type WhisperModelSize = 'tiny' | 'base' | 'small' | 'medium' | 'large';

/**
 * Model preferences interface
 */
export interface ModelPreferences {
  /** Selected Whisper model size */
  whisper_model_size: WhisperModelSize;
  /** URL for the Whisper model */
  whisper_model_url: string;
  /** Filename for the Whisper model */
  whisper_model_filename: string;
  /** URL for the MedLlama model */
  med_llama_url: string;
  /** Filename for the MedLlama model */
  med_llama_filename: string;
  /** Last updated timestamp */
  updated_at: string;
}

/**
 * Downloaded model information
 */
export interface DownloadedModel {
  /** Filename of the model */
  filename: string;
  /** Size in bytes */
  size_bytes: number;
  /** Full path to the model file */
  path: string;
}

/**
 * Whisper model metadata from backend
 */
export interface WhisperModelMetadata {
  /** Value identifier (e.g., "tiny", "base") */
  value: string;
  /** Display label */
  label: string;
  /** Size in megabytes */
  size: number;
  /** Download URL */
  url: string;
  /** Filename */
  file_name: string;
}

/**
 * Runtime binary metadata
 */
export interface RuntimeBinaryMetadata {
  /** Display name */
  name: string;
  /** Download URL */
  url: string;
  /** Filename */
  file_name: string;
  /** Size in megabytes */
  size_mb: number;
}

/**
 * MedLlama model metadata
 */
export interface MedLlamaModelMetadata {
  /** Display name */
  name: string;
  /** Default download URL */
  default_url: string;
  /** Filename */
  file_name: string;
  /** Size in megabytes */
  size_mb: number;
}
