<script lang="ts">
  import { useAuth } from '$lib/hooks/use-auth.svelte.js';
  import Button from '$lib/components/ui/button/button.svelte';
  import LoginForm from '$lib/components/custom/login-form.svelte';
  import AuthStatus from '$lib/components/custom/auth-status.svelte';
  import { ArrowRight } from 'lucide-svelte';

  // Get authentication context
  const auth = useAuth();

  // Reactive state from auth context
  let isAuthenticated = $derived(auth.state.isAuthenticated);
</script>

<svelte:head>
  <title>Medical Note Generator - Authentication Demo</title>
  <meta name="description" content="Medical Note Generator with authentication system demo." />
</svelte:head>

<!-- Authentication Demo Section -->
<section class="bg-muted/30 py-16">
  <div class="container mx-auto px-4 sm:px-6 lg:px-8">
    <div class="mx-auto max-w-6xl">
      <div class="mb-12 text-center">
        <h2 class="text-3xl font-bold tracking-tight sm:text-4xl">Authentication System Demo</h2>
        <p class="mt-4 text-lg text-muted-foreground">Try the login system with any username and password</p>
      </div>

      <!-- Auth Status -->
      <div class="flex justify-center">
        <AuthStatus />
      </div>
    </div>
  </div>
</section>
