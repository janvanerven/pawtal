<script lang="ts">
  import { api } from '$lib/api';
  import type { Media } from '$lib/api/types';
  import { createEventDispatcher, onMount } from 'svelte';

  interface Props {
    open: boolean;
    filter?: 'all' | 'images' | 'icons';
  }

  let { open = $bindable(false), filter = 'all' }: Props = $props();

  const dispatch = createEventDispatcher<{ select: Media; close: void }>();

  let mediaItems = $state<Media[]>([]);
  let loading = $state(false);
  let error = $state('');
  // activeFilter starts from the filter prop but can be changed independently
  let activeFilter = $state<'all' | 'images' | 'icons'>('all');

  $effect(() => {
    activeFilter = filter;
  });

  const filterTabs: { value: 'all' | 'images' | 'icons'; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'images', label: 'Images' },
    { value: 'icons', label: 'Icons' },
  ];

  async function loadMedia() {
    loading = true;
    error = '';
    try {
      const filterParam = activeFilter === 'all' ? undefined : activeFilter;
      const res = await api.admin.listMedia(1, filterParam);
      mediaItems = res.data;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load media';
    } finally {
      loading = false;
    }
  }

  // Load (or reload) whenever the modal opens or the active filter changes
  $effect(() => {
    void activeFilter; // track filter changes
    if (open) loadMedia();
  });

  function selectMedia(item: Media) {
    dispatch('select', item);
    open = false;
  }

  function close() {
    dispatch('close');
    open = false;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={handleBackdropClick}>
    <div class="modal">
      <div class="modal-header">
        <h2>Select Media</h2>
        <button type="button" class="close-btn" onclick={close}>✕</button>
      </div>

      <div class="filter-tabs">
        {#each filterTabs as tab}
          <button
            type="button"
            class="tab-btn"
            class:active={activeFilter === tab.value}
            onclick={() => { activeFilter = tab.value; }}
          >{tab.label}</button>
        {/each}
      </div>

      <div class="modal-body">
        {#if error}
          <p class="error-msg">{error}</p>
        {:else if loading}
          <p class="loading-msg">Loading media...</p>
        {:else if mediaItems.length === 0}
          <p class="empty-msg">No media found.</p>
        {:else}
          <div class="media-grid">
            {#each mediaItems as item (item.id)}
              <button
                type="button"
                class="media-thumb"
                onclick={() => selectMedia(item)}
                title={item.original_filename}
              >
                {#if item.mime_type.startsWith('image/')}
                  <img src="/uploads/{item.filename}" alt={item.alt_text || item.original_filename} />
                {:else}
                  <div class="file-icon">📎</div>
                {/if}
                <span class="thumb-name">{item.original_filename}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-md);
  }

  .modal {
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 100%;
    max-width: 800px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--color-border);
  }

  .modal-header h2 {
    font-size: 1.125rem;
    margin: 0;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    cursor: pointer;
    font-size: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .close-btn:hover {
    background: var(--color-border);
  }

  .filter-tabs {
    display: flex;
    gap: 2px;
    padding: 0 var(--space-lg);
    border-bottom: 2px solid var(--color-border);
  }

  .tab-btn {
    padding: var(--space-sm) var(--space-md);
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .tab-btn.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
  }

  .media-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-sm);
  }

  .media-thumb {
    background: var(--color-bg);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--space-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    text-align: center;
    aspect-ratio: 1;
    overflow: hidden;
  }

  .media-thumb:hover {
    border-color: var(--color-primary);
    box-shadow: var(--shadow-sm);
  }

  .media-thumb img {
    width: 100%;
    height: calc(100% - 24px);
    object-fit: cover;
    border-radius: 4px;
  }

  .file-icon {
    font-size: 2rem;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb-name {
    font-size: 0.65rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
    text-align: center;
  }

  .error-msg { color: var(--color-accent); }
  .loading-msg, .empty-msg { color: var(--color-text-muted); text-align: center; padding: var(--space-xl); }

  @media (max-width: 600px) {
    .media-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
