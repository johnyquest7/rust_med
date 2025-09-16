/**
 * Type definitions for the Medical Note Generator application
 */

/**
 * User information interface
 */
export interface User {
  /** Unique user identifier */
  id: string;
  /** Username for login */
  username: string;
  /** User's full name */
  name: string;
  /** User's email address */
  email: string;
  /** User's medical specialty */
  specialty: string;
  /** When the user was created */
  createdAt: Date;
  /** When the user was last updated */
  updatedAt: Date;
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
 * User registration data interface
 */
export interface RegisterData {
  /** Username for login */
  username: string;
  /** Password for login */
  password: string;
  /** User's full name */
  name: string;
  /** User's email address */
  email: string;
  /** User's medical specialty */
  specialty: string;
}

/**
 * Authentication context methods interface
 */
export interface AuthContext {
  /** Current authentication state */
  state: AuthState;
  /** Login with username and password */
  login: (credentials: LoginCredentials) => Promise<void>;
  /** Logout the current user */
  logout: () => void;
  /** Clear any authentication errors */
  clearError: () => void;
  /** Check if user has specific specialty */
  hasSpecialty: (specialty: string) => boolean;
  /** Check if user is authenticated */
  isAuthenticated: () => boolean;
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
