import type { User, AuthState, RegisterData, AuthContext, AuthResponse, CreateUserRequest, AuthenticateRequest } from '$lib/types.js';
import { browser } from '$app/environment';

declare global {
  interface Window {
    __TAURI__: {
      core: { invoke: (command: string, args?: any) => Promise<any> };
    };
  }
}

/**
 * Authentication context using Svelte 5 runes
 * Provides global authentication state management
 */
class AuthContextClass implements AuthContext {
  // Private state using Svelte 5 runes
  #user = $state<User | null>(null);
  #isLoading = $state(false);
  #error = $state<string | null>(null);

  // Public derived state - using $derived for proper reactivity
  state = $derived({
    user: this.#user,
    isAuthenticated: this.#user !== null,
    isLoading: this.#isLoading,
    error: this.#error
  });

  /**
   * Helper function to invoke Tauri commands
   */
  private async invoke(command: string, args?: any): Promise<any> {
    if (!browser || typeof window.__TAURI__ === 'undefined') {
      throw new Error('Tauri APIs not available');
    }
    return window.__TAURI__.core.invoke(command, args);
  }

  /**
   * Login with password (username is retrieved from auth file)
   * @param password - User password
   */
  async login(password: string): Promise<void> {
    this.#isLoading = true;
    this.#error = null;

    try {
      const request: AuthenticateRequest = { password };
      const response: AuthResponse = await this.invoke('authenticate_user_command', { request });

      if (response.success && response.user) {
        this.#user = response.user;
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
      const response: AuthResponse = await this.invoke('create_user_account_command', { request });

      if (response.success && response.user) {
        this.#user = response.user;
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
      const response: AuthResponse = await this.invoke('check_auth_status');
      
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
        this.#user = user;
      }
    } catch (error) {
      console.error('Failed to initialize auth state:', error);
      // Clear invalid stored data
      localStorage.removeItem('auth_user');
    }
  }
}

// Create singleton instance
const authContext = new AuthContextClass();

// Initialize on module load
if (typeof window !== 'undefined') {
  authContext.initialize();
}

export { authContext };
