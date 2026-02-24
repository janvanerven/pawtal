<script lang="ts">
  import { api } from '$lib/api';
  import type { Article, Category } from '$lib/api/types';
  import { slugify, relativeTime } from '$lib/utils';
  import { goto } from '$app/navigation';
  import { toasts } from '$lib/stores/toasts';
  import RichTextEditor from './RichTextEditor.svelte';
  import MediaPicker from './MediaPicker.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';

  interface Props {
    article?: Article;
    categories: Category[];
  }

  let { article: existingArticle, categories }: Props = $props();

  // Form state
  let title = $state(existingArticle?.title ?? '');
  let slug = $state(existingArticle?.slug ?? '');
  let shortText = $state(existingArticle?.short_text ?? '');
  let content = $state(existingArticle?.content ?? '');
  let status = $state<'draft' | 'published' | 'scheduled'>(
    (existingArticle?.status === 'trashed' ? 'draft' : existingArticle?.status) ?? 'draft'
  );
  let publishAt = $state(existingArticle?.publish_at ?? '');
  let selectedCategoryIds = $state<string[]>([]);
  let coverImageId = $state<string | null>(existingArticle?.cover_image_id ?? null);

  // UI state
  let saving = $state(false);
  let hasUnsavedChanges = $state(false);
  let mediaPickerOpen = $state(false);
  let coverPickerOpen = $state(false);
  let revisionsOpen = $state(false);
  let revisions = $state<import('$lib/api/types').ArticleRevision[]>([]);
  let loadingRevisions = $state(false);

  // Confirm dialogs
  let confirmDeleteOpen = $state(false);
  let confirmRestoreOpen = $state(false);
  let pendingRevisionId = $state('');

  let slugManuallyEdited = $state(!!existingArticle?.slug);

  const coverImageUrl = $derived(
    coverImageId ? `/uploads/${coverImageId}/medium.webp` : null
  );

  function handleTitleInput() {
    hasUnsavedChanges = true;
    if (!slugManuallyEdited) {
      slug = slugify(title);
    }
  }

  function handleSlugInput() {
    slugManuallyEdited = true;
    hasUnsavedChanges = true;
  }

  function handleContentUpdate(html: string) {
    content = html;
    hasUnsavedChanges = true;
  }

  async function save(publish = false) {
    if (!title.trim()) {
      toasts.error('Title is required.');
      return;
    }
    saving = true;

    try {
      const payload = {
        title: title.trim(),
        slug: slug.trim() || slugify(title),
        short_text: shortText,
        content,
        status: publish ? 'published' as const : status,
        publish_at: status === 'scheduled' && publishAt ? publishAt : null,
        category_ids: selectedCategoryIds,
        cover_image_id: coverImageId,
      };

      if (existingArticle) {
        await api.admin.updateArticle(existingArticle.id, payload);
        if (publish) await api.admin.publishArticle(existingArticle.id);
      } else {
        const created = await api.admin.createArticle(payload);
        if (publish) await api.admin.publishArticle(created.id);
        goto(`/admin/articles/${created.id}`);
        return;
      }
      hasUnsavedChanges = false;
      toasts.success('Saved successfully');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Save failed');
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!existingArticle) return;
    try {
      await api.admin.deleteArticle(existingArticle.id);
      toasts.success('Moved to trash');
      goto('/admin/articles');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  async function loadRevisions() {
    if (!existingArticle) return;
    revisionsOpen = !revisionsOpen;
    if (revisionsOpen && revisions.length === 0) {
      loadingRevisions = true;
      try {
        revisions = await api.admin.getArticleRevisions(existingArticle.id);
      } catch {
        // silent
      } finally {
        loadingRevisions = false;
      }
    }
  }

  async function restoreRevision() {
    if (!existingArticle || !pendingRevisionId) return;
    try {
      const restored = await api.admin.restoreArticleRevision(existingArticle.id, pendingRevisionId);
      title = restored.title;
      shortText = restored.short_text;
      content = restored.content;
      toasts.success('Revision restored');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Restore failed');
    }
  }

  function handleImageInsert() {
    mediaPickerOpen = true;
  }

  function handleMediaSelect(event: CustomEvent<import('$lib/api/types').Media>) {
    const media = event.detail;
    window.dispatchEvent(new CustomEvent('rte:insert-image', {
      detail: { src: `/uploads/${media.id}/${media.filename}`, alt: media.alt_text || media.original_filename }
    }));
  }

  function handleCoverSelect(event: CustomEvent<import('$lib/api/types').Media>) {
    const media = event.detail;
    coverImageId = media.id;
    coverPickerOpen = false;
    hasUnsavedChanges = true;
  }
</script>

<div class="editor-layout">
  <div class="editor-header">
    <a href="/admin/articles" class="back-link">← Articles</a>
    <h1 class="editor-title">{existingArticle ? 'Edit Article' : 'New Article'}</h1>
    {#if existingArticle}
      <span class="save-indicator" class:unsaved={hasUnsavedChanges} class:saved={!hasUnsavedChanges}>
        {hasUnsavedChanges ? 'Unsaved changes' : 'Saved'}
      </span>
    {/if}
  </div>

  <div class="editor-body">
    <div class="editor-main">
      <!-- Cover Image -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="cover-image-area" onclick={() => coverPickerOpen = true}>
        {#if coverImageUrl}
          <img src={coverImageUrl} alt="Cover" class="cover-preview" />
          <button
            type="button"
            class="cover-remove"
            onclick={(e: MouseEvent) => { e.stopPropagation(); coverImageId = null; hasUnsavedChanges = true; }}
          >
            Remove cover
          </button>
        {:else}
          <div class="cover-placeholder">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
            <span>Add cover image</span>
          </div>
        {/if}
      </div>

      <div class="field">
        <input
          type="text"
          class="title-input"
          placeholder="Article title..."
          bind:value={title}
          oninput={handleTitleInput}
        />
      </div>

      <div class="field slug-field">
        <label for="slug">Slug</label>
        <div class="slug-prefix">
          <span class="slug-base">/articles/</span>
          <input
            id="slug"
            type="text"
            bind:value={slug}
            oninput={handleSlugInput}
            placeholder="article-slug"
          />
        </div>
      </div>

      <div class="field">
        <label for="short-text">Summary (short text)</label>
        <textarea
          id="short-text"
          bind:value={shortText}
          placeholder="A brief summary of this article..."
          rows="3"
          class="short-text-area"
          oninput={() => hasUnsavedChanges = true}
        ></textarea>
      </div>

      <div class="field">
        <span class="field-label">Content</span>
        <RichTextEditor
          {content}
          onUpdate={handleContentUpdate}
          onInsertImage={handleImageInsert}
        />
      </div>
    </div>

    <aside class="editor-sidebar">
      <div class="sidebar-section card">
        <h3>Status</h3>
        <div class="status-options">
          {#each ['draft', 'published', 'scheduled'] as s}
            <label class="radio-label">
              <input type="radio" name="status" value={s} bind:group={status} />
              <span class="badge badge-{s}">{s}</span>
            </label>
          {/each}
        </div>

        {#if status === 'scheduled'}
          <div class="field" style="margin-top: var(--space-sm)">
            <label for="publish-at">Publish at</label>
            <input
              id="publish-at"
              type="datetime-local"
              bind:value={publishAt}
            />
          </div>
        {/if}
      </div>

      {#if categories.length > 0}
        <div class="sidebar-section card">
          <h3>Categories</h3>
          <div class="category-list">
            {#each categories as cat (cat.id)}
              <label class="checkbox-label">
                <input
                  type="checkbox"
                  value={cat.id}
                  bind:group={selectedCategoryIds}
                />
                {cat.name}
              </label>
            {/each}
          </div>
        </div>
      {/if}

      <div class="sidebar-section card">
        <h3>Actions</h3>
        <div class="action-btns">
          <button
            type="button"
            class="btn btn-ghost"
            disabled={saving}
            onclick={() => save(false)}
          >
            {saving ? 'Saving...' : 'Save Draft'}
          </button>
          <button
            type="button"
            class="btn btn-primary"
            disabled={saving}
            onclick={() => save(true)}
          >
            {saving ? 'Publishing...' : 'Publish'}
          </button>
          {#if existingArticle}
            <button
              type="button"
              class="btn btn-danger"
              onclick={() => confirmDeleteOpen = true}
            >
              Move to Trash
            </button>
          {/if}
        </div>
      </div>

      {#if existingArticle}
        <div class="sidebar-section card">
          <button
            type="button"
            class="revisions-toggle"
            onclick={loadRevisions}
          >
            {revisionsOpen ? '▼' : '▶'} Revision History
          </button>

          {#if revisionsOpen}
            {#if loadingRevisions}
              <p class="muted-text">Loading...</p>
            {:else if revisions.length === 0}
              <p class="muted-text">No revisions saved yet.</p>
            {:else}
              <div class="revisions-list">
                {#each revisions as rev (rev.id)}
                  <div class="revision-entry">
                    <div class="revision-info">
                      <span class="revision-title">{rev.title}</span>
                      <span class="revision-time">{relativeTime(rev.created_at)}</span>
                    </div>
                    <button
                      type="button"
                      class="btn btn-ghost"
                      style="font-size: 0.75rem; padding: 2px 8px;"
                      onclick={() => { pendingRevisionId = rev.id; confirmRestoreOpen = true; }}
                    >Restore</button>
                  </div>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      {/if}
    </aside>
  </div>
</div>

<MediaPicker bind:open={mediaPickerOpen} on:select={handleMediaSelect} />
<MediaPicker bind:open={coverPickerOpen} on:select={handleCoverSelect} />

<ConfirmDialog
  bind:open={confirmDeleteOpen}
  title="Move to Trash"
  message="Are you sure you want to move this article to trash?"
  confirmLabel="Move to Trash"
  variant="danger"
  onConfirm={handleDelete}
/>

<ConfirmDialog
  bind:open={confirmRestoreOpen}
  title="Restore Revision"
  message="Restore this revision? Unsaved changes will be lost."
  confirmLabel="Restore"
  onConfirm={restoreRevision}
/>

<style>
  .editor-layout { max-width: 1200px; }

  .editor-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    margin-bottom: var(--space-lg);
  }

  .back-link { color: var(--color-text-muted); font-size: 0.875rem; text-decoration: none; white-space: nowrap; }
  .back-link:hover { color: var(--color-text); }
  .editor-title { font-size: 1.25rem; margin: 0; }

  .save-indicator {
    margin-left: auto;
    font-size: 0.75rem;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .save-indicator::before {
    content: '';
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .save-indicator.saved { color: var(--color-text-light); }
  .save-indicator.saved::before { background: #7BA68C; }
  .save-indicator.unsaved { color: var(--color-primary); }
  .save-indicator.unsaved::before { background: var(--color-primary); }

  /* Cover image area */
  .cover-image-area {
    width: 100%;
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    transition: border-color var(--transition-fast);
    position: relative;
    background: transparent;
    padding: 0;
    text-align: left;
  }
  .cover-image-area:hover { border-color: var(--color-primary); }

  .cover-preview {
    width: 100%;
    height: 200px;
    object-fit: cover;
    display: block;
  }

  .cover-remove {
    position: absolute;
    top: var(--space-sm);
    right: var(--space-sm);
    padding: var(--space-xs) var(--space-sm);
    background: rgba(0,0,0,0.6);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    cursor: pointer;
  }
  .cover-remove:hover { background: rgba(0,0,0,0.8); }

  .cover-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-xl);
    color: var(--color-text-light);
    font-size: 0.85rem;
  }

  .editor-body {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: var(--space-lg);
    align-items: start;
  }

  .editor-main { display: flex; flex-direction: column; gap: var(--space-md); min-width: 0; }

  .field { display: flex; flex-direction: column; gap: var(--space-xs); }
  .field label,
  .field-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .title-input {
    font-family: var(--font-heading);
    font-size: 1.75rem;
    font-weight: 700;
    border: none;
    border-bottom: 2px solid var(--color-border);
    border-radius: 0;
    padding: var(--space-sm) 0;
    background: transparent;
    color: var(--color-text);
    width: 100%;
  }
  .title-input:focus { outline: none; border-bottom-color: var(--color-primary); box-shadow: none; }
  .title-input::placeholder { color: var(--color-text-light); }

  .slug-prefix {
    display: flex;
    align-items: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--color-surface);
  }
  .slug-base {
    padding: var(--space-sm) var(--space-sm) var(--space-sm) var(--space-md);
    color: var(--color-text-muted);
    background: var(--color-bg);
    border-right: 1px solid var(--color-border);
    font-size: 0.9rem;
    flex-shrink: 0;
  }
  .slug-prefix input { border: none; border-radius: 0; flex: 1; padding: var(--space-sm) var(--space-md); font-size: 0.9rem; }
  .slug-prefix input:focus { box-shadow: none; outline: none; border-color: transparent; }

  .short-text-area {
    resize: vertical;
    min-height: 80px;
    font-size: 0.95rem;
    line-height: 1.6;
  }

  .editor-sidebar { display: flex; flex-direction: column; gap: var(--space-md); position: sticky; top: var(--space-md); }
  .sidebar-section { padding: var(--space-md); }
  .sidebar-section h3 {
    font-size: 0.8rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin-bottom: var(--space-sm);
  }

  .status-options { display: flex; flex-direction: column; gap: var(--space-xs); }
  .radio-label { display: flex; align-items: center; gap: var(--space-sm); cursor: pointer; padding: var(--space-xs) 0; }
  .radio-label input[type="radio"] { border: none; padding: 0; width: auto; margin: 0; cursor: pointer; }

  .category-list { display: flex; flex-direction: column; gap: var(--space-xs); }
  .checkbox-label { display: flex; align-items: center; gap: var(--space-sm); font-size: 0.875rem; cursor: pointer; }
  .checkbox-label input[type="checkbox"] { border: none; padding: 0; width: auto; margin: 0; cursor: pointer; }

  .action-btns { display: flex; flex-direction: column; gap: var(--space-sm); }
  .action-btns .btn { width: 100%; justify-content: center; }

  .revisions-toggle {
    background: none; border: none; font-size: 0.8rem; font-weight: 600; color: var(--color-text-muted);
    cursor: pointer; padding: 0; margin-bottom: var(--space-sm); width: 100%; text-align: left;
  }
  .revisions-toggle:hover { color: var(--color-text); }

  .revisions-list { display: flex; flex-direction: column; gap: var(--space-xs); margin-top: var(--space-sm); }
  .revision-entry {
    display: flex; align-items: center; justify-content: space-between; gap: var(--space-xs);
    padding: var(--space-xs) 0; border-bottom: 1px solid var(--color-border);
  }
  .revision-entry:last-child { border-bottom: none; }
  .revision-info { display: flex; flex-direction: column; min-width: 0; }
  .revision-title { font-size: 0.8rem; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .revision-time { font-size: 0.7rem; color: var(--color-text-muted); }
  .muted-text { font-size: 0.8rem; color: var(--color-text-muted); margin-top: var(--space-sm); }

  @media (max-width: 900px) {
    .editor-body { grid-template-columns: 1fr; }
    .editor-sidebar { position: static; }
  }
</style>
