<script lang="ts">
  import { cn } from '$lib/utils';

  let { value = 0, class: className, ...restProps }: { value?: number; class?: string } = $props();

  const clampedValue = $derived(Math.min(100, Math.max(0, value)));
</script>

<div class={cn('relative h-4 w-full overflow-hidden rounded-full bg-secondary', className)} {...restProps}>
  <div
    class="h-full w-full flex-1 bg-primary transition-all duration-300 ease-in-out"
    style="transform: translateX(-{100 - clampedValue}%)"
  ></div>
</div>
