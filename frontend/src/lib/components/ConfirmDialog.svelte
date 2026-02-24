<script lang="ts">
  import { fly, fade } from 'svelte/transition';

  interface Props {
    open: boolean;
    title?: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: 'danger' | 'default';
    onConfirm: () => void;
    onCancel?: () => void;
  }

  let {
    open = $bindable(false),
    title = 'Confirm',
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    variant = 'default',
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleConfirm() {
    onConfirm();
    open = false;
  }

  function handleCancel() {
    onCancel?.();
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleCancel();
  }
</script>

<svelte:window onkeydown={open ? handleKeydown : undefined} />

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog-overlay" transition:fade={{ duration: 200 }} onclick={handleCancel}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="dialog-content" transition:fly={{ y: 20, duration: 250 }} onclick={(e) => e.stopPropagation()}>
      <h3 class="dialog-title">{title}</h3>
      <p class="dialog-message">{message}</p>
      <div class="dialog-actions">
        <button class="btn btn-ghost" onclick={handleCancel}>{cancelLabel}</button>
        <button class="btn {variant === 'danger' ? 'btn-danger' : 'btn-primary'}" onclick={handleConfirm}>
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .dialog-content {
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    padding: var(--space-xl);
    max-width: 440px;
    width: 90%;
    box-shadow: var(--shadow-lg);
  }

  .dialog-title {
    font-size: 1.125rem;
    margin-bottom: var(--space-sm);
  }

  .dialog-message {
    color: var(--color-text-muted);
    font-size: 0.9rem;
    line-height: 1.6;
    margin-bottom: var(--space-lg);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
</style>
