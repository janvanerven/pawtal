<script lang="ts">
  import { toasts } from '$lib/stores/toasts';
  import { fly } from 'svelte/transition';
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div
      class="toast toast-{toast.type}"
      transition:fly={{ x: 300, duration: 300 }}
    >
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" aria-label="Dismiss notification" onclick={() => toasts.remove(toast.id)}>&times;</button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: var(--space-lg);
    right: var(--space-lg);
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    font-size: 0.875rem;
    font-weight: 500;
    backdrop-filter: blur(12px);
  }

  .toast-success {
    background: rgba(76, 175, 80, 0.95);
    color: white;
  }
  .toast-error {
    background: rgba(232, 93, 93, 0.95);
    color: white;
  }
  .toast-info {
    background: var(--color-surface-elevated);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }

  .toast-message { flex: 1; }

  .toast-close {
    font-size: 1.25rem;
    line-height: 1;
    opacity: 0.7;
    color: inherit;
    padding: 0;
    background: none;
  }
  .toast-close:hover { opacity: 1; }
</style>
