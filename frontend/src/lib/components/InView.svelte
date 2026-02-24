<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    threshold?: number;
    rootMargin?: string;
    once?: boolean;
    class?: string;
    delay?: number;
    children: Snippet;
  }

  let { threshold = 0.1, rootMargin = '0px', once = true, class: className = '', delay = 0, children }: Props = $props();

  let element: HTMLElement;
  let visible = $state(false);

  onMount(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          if (delay > 0) {
            setTimeout(() => { visible = true; }, delay);
          } else {
            visible = true;
          }
          if (once) observer.unobserve(element);
        } else if (!once) {
          visible = false;
        }
      },
      { threshold, rootMargin }
    );
    observer.observe(element);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={element}
  class="in-view {className}"
  class:visible
>
  {@render children()}
</div>

<style>
  .in-view {
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.6s ease-out, transform 0.6s ease-out;
  }
  .in-view.visible {
    opacity: 1;
    transform: translateY(0);
  }
</style>
