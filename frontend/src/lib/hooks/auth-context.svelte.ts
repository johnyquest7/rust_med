import type { User, AuthState, LoginCredentials, AuthContext } from '$lib/types.js';

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
   * Login with username and password
   * @param credentials - Login credentials
   */
  async login(credentials: LoginCredentials): Promise<void> {
    this.#isLoading = true;
    this.#error = null;

    try {
      // TODO: Replace with actual authentication logic
      // For now, we'll simulate a login process
      await this.#simulateLogin(credentials);

      // Create a mock user for now
      const user: User = {
        id: '1',
        username: credentials.username,
        name: 'Dr. ' + credentials.username.charAt(0).toUpperCase() + credentials.username.slice(1),
        email: `${credentials.username}@medicalnotes.com`,
        specialty: 'General Practitioner',
        createdAt: new Date(),
        updatedAt: new Date()
      };

      this.#user = user;

      // Store in localStorage for persistence
      localStorage.setItem('auth_user', JSON.stringify(user));
    } catch (error) {
      this.#error = error instanceof Error ? error.message : 'Login failed';
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
   * Check if user has specific specialty
   * @param specialty - Specialty to check
   * @returns True if user has the specialty
   */
  hasSpecialty(specialty: string): boolean {
    return this.#user?.specialty === specialty;
  }

  /**
   * Check if user is authenticated
   * @returns True if user is authenticated
   */
  isAuthenticated(): boolean {
    return this.#user !== null;
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
        // Convert date strings back to Date objects
        user.createdAt = new Date(user.createdAt);
        user.updatedAt = new Date(user.updatedAt);
        this.#user = user;
      }
    } catch (error) {
      console.error('Failed to initialize auth state:', error);
      // Clear invalid stored data
      localStorage.removeItem('auth_user');
    }
  }

  /**
   * Simulate login process
   * @param credentials - Login credentials
   * @private
   */
  async #simulateLogin(credentials: LoginCredentials): Promise<void> {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // For now, accept any username/password combination
    // TODO: Replace with actual authentication logic
    if (!credentials.username || !credentials.password) {
      throw new Error('Username and password are required');
    }

    // Simulate some validation
    if (credentials.username.length < 3) {
      throw new Error('Username must be at least 3 characters long');
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
