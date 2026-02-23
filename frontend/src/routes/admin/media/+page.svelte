<script lang="ts">
  import { api } from '$lib/api';
  import type { Media } from '$lib/api/types';
  import { formatFileSize, formatDate } from '$lib/utils';

  type Filter = 'all' | 'images' | 'icons';

  let filter = $state<Filter>('all');
  let mediaItems = $state<Media[]>([]);
  let loading = $state(false);
  let uploading = $state(false);
  let error = $state('');
  let selectedItem = $state<Media | null>(null);
  let altTextInput = $state('');
  let dragOver = $state(false);
  let fileInput: HTMLInputElement;

  const filterTabs: { value: Filter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'images', label: 'Images' },
    { value: 'icons', label: 'Icons' },
  ];

  async function loadMedia() {
    loading = true;
    error = '';
    try {
      const filterParam = filter === 'all' ? undefined : filter;
      const res = await api.admin.listMedia(1, filterParam);
      mediaItems = res.data;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load media';
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void loadMedia();
  });

  async function uploadFiles(files: FileList | File[]) {
    uploading = true;
    error = '';
    const fileArray = Array.from(files);

    for (const file of fileArray) {
      try {
        const isIcon = file.type === 'image/svg+xml' || file.name.endsWith('.svg');
        await api.admin.uploadMedia(file, isIcon);
      } catch (e) {
        error = `Failed to upload ${file.name}: ${e instanceof Error ? e.message : 'Unknown error'}`;
      }
    }

    uploading = false;
    loadMedia();
  }

  function handleFileInput(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files) uploadFiles(input.files);
    input.value = '';
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    if (e.dataTransfer?.files) uploadFiles(e.dataTransfer.files);
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
  }

  function handleDragLeave() {
    dragOver = false;
  }

  function selectItem(item: Media) {
    selectedItem = item;
    altTextInput = item.alt_text;
  }

  async function deleteItem(item: Media) {
    if (!confirm(`Delete "${item.original_filename}"?`)) return;
    try {
      await api.admin.deleteMedia(item.id);
      mediaItems = mediaItems.filter(m => m.id !== item.id);
      if (selectedItem?.id === item.id) selectedItem = null;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Delete failed';
    }
  }
</script>

<svelte:head>
  <title>Media — Pawtal CMS</title>
</svelte:head>

<div class="media-page">
  <div class="page-header">
    <h1>Media Library</h1>
  </div>

  <!-- Upload zone -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="upload-zone card"
    class:drag-over={dragOver}
    ondrop={handleDrop}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
  >
    {#if uploading}
      <p class="upload-text">Uploading...</p>
    {:else}
      <p class="upload-text">Drag and drop files here</p>
      <p class="upload-subtext">or</p>
      <button
        type="button"
        class="btn btn-primary"
        onclick={() => fileInput.click()}
      >Choose Files</button>
    {/if}
    <input
      type="file"
      multiple
      accept="image/*,video/*"
      style="display: none"
      bind:this={fileInput}
      onchange={handleFileInput}
    />
  </div>

  <!-- Filter tabs -->
  <div class="filter-tabs">
    {#each filterTabs as tab}
      <button
        class="tab-btn"
        class:active={filter === tab.value}
        onclick={() => { filter = tab.value; }}
      >{tab.label}</button>
    {/each}
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  <div class="media-layout">
    <!-- Grid -->
    <div class="media-grid-area">
      {#if loading}
        <p class="muted-text">Loading media...</p>
      {:else if mediaItems.length === 0}
        <p class="muted-text">No media uploaded yet.</p>
      {:else}
        <div class="media-grid">
          {#each mediaItems as item (item.id)}
            <button
              type="button"
              class="media-thumb"
              class:selected={selectedItem?.id === item.id}
              onclick={() => selectItem(item)}
            >
              {#if item.mime_type.startsWith('image/')}
                <img
                  src="/uploads/{item.filename}"
                  alt={item.alt_text || item.original_filename}
                  loading="lazy"
                />
              {:else}
                <div class="file-icon">📎</div>
              {/if}
              <span class="thumb-name">{item.original_filename}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Detail panel -->
    {#if selectedItem}
      <aside class="detail-panel card">
        <div class="detail-preview">
          {#if selectedItem.mime_type.startsWith('image/')}
            <img
              src="/uploads/{selectedItem.filename}"
              alt={selectedItem.alt_text || selectedItem.original_filename}
            />
          {:else}
            <div class="detail-file-icon">📎</div>
          {/if}
        </div>

        <div class="detail-info">
          <h3>{selectedItem.original_filename}</h3>
          <dl class="detail-meta">
            <dt>Type</dt><dd>{selectedItem.mime_type}</dd>
            <dt>Size</dt><dd>{formatFileSize(selectedItem.size_bytes)}</dd>
            {#if selectedItem.width && selectedItem.height}
              <dt>Dimensions</dt><dd>{selectedItem.width} × {selectedItem.height}</dd>
            {/if}
            <dt>Uploaded</dt><dd>{formatDate(selectedItem.created_at)}</dd>
          </dl>

          <div class="detail-url">
            <label for="media-url">URL</label>
            <input
              id="media-url"
              type="text"
              readonly
              value="/uploads/{selectedItem.filename}"
              onclick={(e) => (e.target as HTMLInputElement).select()}
            />
          </div>

          <button
            type="button"
            class="btn btn-danger"
            style="width: 100%; justify-content: center; margin-top: var(--space-sm);"
            onclick={() => selectedItem && deleteItem(selectedItem)}
          >Delete</button>
        </div>
      </aside>
    {/if}
  </div>
</div>

<style>
  .media-page { max-width: 1200px; }

  .page-header { margin-bottom: var(--space-lg); }

  .upload-zone {
    text-align: center;
    padding: var(--space-2xl);
    border: 2px dashed var(--color-border);
    background: var(--color-bg);
    margin-bottom: var(--space-lg);
    transition: all var(--transition-fast);
  }

  .upload-zone.drag-over {
    border-color: var(--color-primary);
    background: rgba(232, 146, 74, 0.05);
  }

  .upload-text { font-size: 1rem; font-weight: 500; color: var(--color-text); margin-bottom: var(--space-xs); }
  .upload-subtext { font-size: 0.875rem; color: var(--color-text-muted); margin-bottom: var(--space-md); }

  .filter-tabs {
    display: flex; gap: 2px; margin-bottom: var(--space-lg);
    border-bottom: 2px solid var(--color-border);
  }

  .tab-btn {
    padding: var(--space-sm) var(--space-md); background: none; border: none;
    border-bottom: 2px solid transparent; margin-bottom: -2px; font-size: 0.875rem;
    font-weight: 500; color: var(--color-text-muted); cursor: pointer; transition: all var(--transition-fast);
  }
  .tab-btn:hover { color: var(--color-text); }
  .tab-btn.active { color: var(--color-primary); border-bottom-color: var(--color-primary); }

  .error-banner { background: #FFEBEE; color: var(--color-accent); padding: var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); }

  .media-layout {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: var(--space-lg);
    align-items: start;
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
    aspect-ratio: 1;
    overflow: hidden;
  }

  .media-thumb:hover { border-color: var(--color-primary); }
  .media-thumb.selected { border-color: var(--color-primary); box-shadow: 0 0 0 2px rgba(232, 146, 74, 0.3); }

  .media-thumb img {
    width: 100%;
    height: calc(100% - 20px);
    object-fit: cover;
    border-radius: 4px;
  }

  .file-icon { font-size: 2rem; flex: 1; display: flex; align-items: center; justify-content: center; }
  .thumb-name { font-size: 0.65rem; color: var(--color-text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; width: 100%; text-align: center; }

  /* Detail panel */
  .detail-panel { padding: var(--space-md); }
  .detail-preview { margin-bottom: var(--space-md); text-align: center; }
  .detail-preview img { max-width: 100%; max-height: 200px; object-fit: contain; border-radius: var(--radius-sm); }
  .detail-file-icon { font-size: 4rem; padding: var(--space-lg); }

  .detail-info h3 { font-size: 0.9rem; word-break: break-all; margin-bottom: var(--space-sm); }

  .detail-meta { display: grid; grid-template-columns: auto 1fr; gap: var(--space-xs) var(--space-md); font-size: 0.8rem; margin-bottom: var(--space-md); }
  .detail-meta dt { color: var(--color-text-muted); font-weight: 600; }
  .detail-meta dd { color: var(--color-text); }

  .detail-url label { display: block; font-size: 0.75rem; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: var(--space-xs); }
  .detail-url input { width: 100%; font-size: 0.75rem; }

  .muted-text { color: var(--color-text-muted); padding: var(--space-lg) 0; }

  @media (max-width: 900px) {
    .media-layout { grid-template-columns: 1fr; }
    .media-grid { grid-template-columns: repeat(2, 1fr); }
  }
</style>
