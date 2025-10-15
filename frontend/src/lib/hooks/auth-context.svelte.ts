import type {
  User,
  AuthState,
  RegisterData,
  AuthContext,
  AuthResponse,
  CreateUserRequest,
  AuthenticateRequest
} from '$lib/types.js';
import { browser } from '$app/environment';
import { tauriService } from '$lib/tauriService.js';

/**
 * Authentication context using Svelte 5 runes
 * Provides global authentication state management
 */
class AuthContextClass implements AuthContext {
  // Private
  #user = $state<User | null>(null);
  #isLoading = $state(false);
  #error = $state<string | null>(null);
  #password = $state<string | null>(null); // Store password temporarily for encryption

  // Public
  state = $derived({
    user: this.#user,
    isAuthenticated: this.#user !== null && this.#password !== null && this.#password !== '',
    isLoading: this.#isLoading,
    error: this.#error
  });

  /**
   * Login with password (username is retrieved from auth file)
   * @param password - User password
   */
  async login(password: string): Promise<void> {
    this.#isLoading = true;
    this.#error = null;

    try {
      const request: AuthenticateRequest = { password };
      const response: AuthResponse = await tauriService.authenticateUser(request);

      if (response.success && response.user) {
        this.#user = response.user;
        this.#password = password; // Store password temporarily for encryption
        // Store in localStorage for persistence
        localStorage.setItem('auth_user', JSON.stringify(response.user));
      } else {
        throw new Error(response.message);
      }
    } catch (error) {
      this.#error = error instanceof Error ? error.message : 'Login failed';
      throw error;
    } finally {
      this.#isLoading = false;
    }
  }

  /**
   * Register new user account
   * @param data - Registration data
   */
  async register(data: RegisterData): Promise<void> {
    this.#isLoading = true;
    this.#error = null;

    try {
      const request: CreateUserRequest = {
        username: data.username,
        password: data.password
      };
      const response: AuthResponse = await tauriService.createUserAccount(request);

      if (response.success && response.user) {
        this.#user = response.user;
        this.#password = data.password; // Store password temporarily for encryption
        // Store in localStorage for persistence
        localStorage.setItem('auth_user', JSON.stringify(response.user));
      } else {
        throw new Error(response.message);
      }
    } catch (error) {
      this.#error = error instanceof Error ? error.message : 'Registration failed';
      throw error;
    } finally {
      this.#isLoading = false;
    }
  }

  /**
   * Logout the current user
   */
  logout(): void {
    this.#user = null;
    this.#password = null; // Clear password from memory
    this.#error = null;
    localStorage.removeItem('auth_user');
  }

  /**
   * Clear any authentication errors
   */
  clearError(): void {
    this.#error = null;
  }

  /**
   * Check if user is authenticated
   * @returns True if user is authenticated
   */
  isAuthenticated(): boolean {
    return this.#user !== null;
  }

  /**
   * Check authentication status on app startup
   */
  async checkAuthStatus(): Promise<void> {
    this.#isLoading = true;
    this.#error = null;

    try {
      const response: AuthResponse = await tauriService.checkAuthStatus();

      if (response.success && response.user) {
        this.#user = response.user;
        // Store in localStorage for persistence
        localStorage.setItem('auth_user', JSON.stringify(response.user));
      } else {
        // No auth file exists or user not authenticated
        this.#user = null;
        localStorage.removeItem('auth_user');
      }
    } catch (error) {
      console.error('Failed to check auth status:', error);
      this.#user = null;
      localStorage.removeItem('auth_user');
    } finally {
      this.#isLoading = false;
    }
  }

  /**
   * Initialize authentication state from localStorage
   * Should be called on app startup
   */
  initialize(): void {
    try {
      const storedUser = localStorage.getItem('auth_user');
      if (storedUser) {
        const user = JSON.parse(storedUser) as User;
        // Don't restore user without password - user needs to log in again
        // this.#user = user;
        console.log('Found stored user but password not available - user needs to log in');
        // Clear stored user to force re-login
        localStorage.removeItem('auth_user');
      }
    } catch (error) {
      console.error('Failed to initialize auth state:', error);
      // Clear invalid stored data
      localStorage.removeItem('auth_user');
    }
  }

  /**
   * Get the current password for encryption (only available when authenticated)
   * @returns The password if available, null otherwise
   */
  getPassword(): string | null {
    return this.#password;
  }
}

// Create singleton instance
const authContext = new AuthContextClass();

// Initialize on module load
if (typeof window !== 'undefined') {
  authContext.initialize();
}

export { authContext };
