<script lang="ts">
  import { Toaster as Sonner, type ToasterProps as SonnerProps } from 'svelte-sonner';
  import { mode } from 'mode-watcher';

  let { ...restProps }: SonnerProps = $props();
</script>

<Sonner
  theme={mode.current}
  class="toaster group"
  toastOptions={{
    classes: {
      toast: 'bg-popover text-popover-foreground border-border',
      title: 'text-popover-foreground font-semibold',
      description: 'text-popover-foreground/90',
      actionButton: 'bg-primary text-primary-foreground',
      cancelButton: 'bg-muted text-muted-foreground',
      error: 'bg-destructive text-destructive-foreground border-destructive',
      success: 'bg-primary text-primary-foreground border-primary',
      warning: 'bg-secondary text-secondary-foreground border-secondary',
      info: 'bg-popover text-popover-foreground border-border'
    }
  }}
  {...restProps}
/>
