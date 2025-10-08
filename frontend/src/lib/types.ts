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
