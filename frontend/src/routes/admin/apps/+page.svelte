<script lang="ts">
  import { api } from '$lib/api';
  import type { App, Media, Page } from '$lib/api/types';
  import { onMount } from 'svelte';
  import MediaPicker from '$lib/components/MediaPicker.svelte';

  let apps = $state<App[]>([]);
  let pages = $state<Page[]>([]);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state('');

  // Form state
  let showForm = $state(false);
  let editingApp = $state<App | null>(null);
  let formName = $state('');
  let formDescription = $state('');
  let formIconId = $state<string | null>(null);
  let formIconUrl = $state('');
  let formLinkType = $state<'url' | 'page'>('url');
  let formUrl = $state('');
  let formPageId = $state<string | null>(null);
  let mediaPickerOpen = $state(false);

  async function loadData() {
    loading = true;
    try {
      const [appsRes, pagesRes] = await Promise.allSettled([
        api.admin.listApps(),
        api.admin.listPages(1),
      ]);
      if (appsRes.status === 'fulfilled') apps = appsRes.value.data;
      if (pagesRes.status === 'fulfilled') pages = pagesRes.value.data;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load';
    } finally {
      loading = false;
    }
  }

  onMount(() => loadData());

  function openAddForm() {
    editingApp = null;
    formName = '';
    formDescription = '';
    formIconId = null;
    formIconUrl = '';
    formLinkType = 'url';
    formUrl = '';
    formPageId = null;
    showForm = true;
  }

  function openEditForm(app: App) {
    editingApp = app;
    formName = app.name;
    formDescription = app.description;
    formIconId = app.icon_id;
    formIconUrl = app.icon_id ? `/uploads/${app.icon_id}` : '';
    formLinkType = app.page_id ? 'page' : 'url';
    formUrl = app.url ?? '';
    formPageId = app.page_id;
    showForm = true;
  }

  async function saveApp() {
    if (!formName.trim()) { error = 'App name is required.'; return; }
    saving = true;
    error = '';
    try {
      const payload = {
        name: formName.trim(),
        description: formDescription,
        icon_id: formIconId || null,
        url: formLinkType === 'url' ? (formUrl || null) : null,
        page_id: formLinkType === 'page' ? (formPageId || null) : null,
      };

      if (editingApp) {
        const updated = await api.admin.updateApp(editingApp.id, payload);
        apps = apps.map(a => a.id === updated.id ? updated : a);
      } else {
        const created = await api.admin.createApp(payload);
        apps = [...apps, created];
      }
      showForm = false;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Save failed';
    } finally {
      saving = false;
    }
  }

  async function deleteApp(app: App) {
    if (!confirm(`Delete "${app.name}"?`)) return;
    try {
      await api.admin.deleteApp(app.id);
      apps = apps.filter(a => a.id !== app.id);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Delete failed';
    }
  }

  async function moveUp(index: number) {
    if (index === 0) return;
    const newApps = [...apps];
    [newApps[index - 1], newApps[index]] = [newApps[index], newApps[index - 1]];
    apps = newApps;
    await api.admin.reorderApps(apps.map(a => a.id));
  }

  async function moveDown(index: number) {
    if (index >= apps.length - 1) return;
    const newApps = [...apps];
    [newApps[index], newApps[index + 1]] = [newApps[index + 1], newApps[index]];
    apps = newApps;
    await api.admin.reorderApps(apps.map(a => a.id));
  }

  function handleIconSelect(event: CustomEvent<Media>) {
    const media = event.detail;
    formIconId = media.id;
    formIconUrl = `/uploads/${media.filename}`;
  }
</script>

<svelte:head>
  <title>Apps — Pawtal CMS</title>
</svelte:head>

<div class="apps-page">
  <div class="page-header">
    <h1>App Catalogue</h1>
    <button class="btn btn-primary" onclick={openAddForm}>+ Add App</button>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if loading}
    <p class="muted-text">Loading...</p>
  {:else if apps.length === 0}
    <div class="card empty-state">
      <p>No apps yet. Add your first app!</p>
    </div>
  {:else}
    <div class="apps-list">
      {#each apps as app, i (app.id)}
        <div class="app-item card">
          <div class="app-icon">
            {#if app.icon_id}
              <img src="/uploads/{app.icon_id}" alt="{app.name} icon" />
            {:else}
              <div class="app-icon-placeholder">⚡</div>
            {/if}
          </div>
          <div class="app-info">
            <span class="app-name">{app.name}</span>
            <span class="app-desc">{app.description}</span>
            {#if app.url}
              <span class="app-link">🔗 {app.url}</span>
            {:else if app.page_id}
              <span class="app-link">📄 Page: {app.page_id}</span>
            {/if}
          </div>
          <div class="app-actions">
            <div class="order-btns">
              <button class="btn btn-ghost" onclick={() => moveUp(i)} disabled={i === 0} title="Move up">↑</button>
              <button class="btn btn-ghost" onclick={() => moveDown(i)} disabled={i >= apps.length - 1} title="Move down">↓</button>
            </div>
            <button class="btn btn-ghost" onclick={() => openEditForm(app)}>Edit</button>
            <button class="btn btn-danger" onclick={() => deleteApp(app)}>Delete</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add/Edit form modal -->
{#if showForm}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) showForm = false; }}>
    <div class="modal card">
      <div class="modal-header">
        <h2>{editingApp ? 'Edit App' : 'Add App'}</h2>
        <button type="button" class="close-btn" onclick={() => showForm = false}>✕</button>
      </div>

      {#if error}
        <div class="alert alert-error">{error}</div>
      {/if}

      <div class="form-fields">
        <div class="field">
          <label for="app-name">Name *</label>
          <input id="app-name" type="text" bind:value={formName} placeholder="App name" />
        </div>

        <div class="field">
          <label for="app-desc">Description</label>
          <textarea id="app-desc" bind:value={formDescription} placeholder="Short description" rows="2"></textarea>
        </div>

        <div class="field">
          <span class="field-label">Icon</span>
          <div class="icon-row">
            {#if formIconUrl}
              <img src={formIconUrl} alt="icon" class="icon-preview" />
            {/if}
            <button type="button" class="btn btn-ghost" onclick={() => mediaPickerOpen = true}>
              {formIconId ? 'Change Icon' : 'Select Icon'}
            </button>
            {#if formIconId}
              <button type="button" class="btn btn-ghost" onclick={() => { formIconId = null; formIconUrl = ''; }}>Remove</button>
            {/if}
          </div>
        </div>

        <div class="field">
          <span class="field-label">Link Type</span>
          <div class="radio-row">
            <label class="radio-label">
              <input type="radio" name="link-type" value="url" bind:group={formLinkType} />
              External URL
            </label>
            <label class="radio-label">
              <input type="radio" name="link-type" value="page" bind:group={formLinkType} />
              Internal Page
            </label>
          </div>
        </div>

        {#if formLinkType === 'url'}
          <div class="field">
            <label for="app-url">URL</label>
            <input id="app-url" type="url" bind:value={formUrl} placeholder="https://example.com" />
          </div>
        {:else}
          <div class="field">
            <label for="app-page">Page</label>
            <select id="app-page" bind:value={formPageId}>
              <option value={null}>-- Select a page --</option>
              {#each pages as p (p.id)}
                <option value={p.id}>{p.title}</option>
              {/each}
            </select>
          </div>
        {/if}
      </div>

      <div class="modal-actions">
        <button type="button" class="btn btn-ghost" onclick={() => showForm = false}>Cancel</button>
        <button type="button" class="btn btn-primary" disabled={saving} onclick={saveApp}>
          {saving ? 'Saving...' : 'Save App'}
        </button>
      </div>
    </div>
  </div>
{/if}

<MediaPicker bind:open={mediaPickerOpen} filter="icons" on:select={handleIconSelect} />

<style>
  .apps-page { max-width: 800px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-lg); }
  .error-banner { background: #FFEBEE; color: var(--color-accent); padding: var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); }
  .muted-text { color: var(--color-text-muted); }
  .empty-state { text-align: center; color: var(--color-text-muted); padding: var(--space-2xl); }

  .apps-list { display: flex; flex-direction: column; gap: var(--space-sm); }

  .app-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
  }

  .app-icon {
    width: 48px;
    height: 48px;
    flex-shrink: 0;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--color-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
  }

  .app-icon img { width: 100%; height: 100%; object-fit: cover; }
  .app-icon-placeholder { font-size: 1.5rem; }

  .app-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .app-name { font-weight: 600; font-size: 0.95rem; }
  .app-desc { font-size: 0.8rem; color: var(--color-text-muted); }
  .app-link { font-size: 0.75rem; color: var(--color-primary); }

  .app-actions { display: flex; align-items: center; gap: var(--space-xs); flex-shrink: 0; }
  .order-btns { display: flex; flex-direction: column; gap: 2px; }
  .order-btns .btn { padding: 2px 6px; font-size: 0.75rem; }

  /* Modal */
  .modal-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 1000;
    display: flex; align-items: center; justify-content: center; padding: var(--space-md);
  }

  .modal {
    width: 100%; max-width: 500px; max-height: 90vh; overflow-y: auto;
  }

  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: var(--space-lg);
  }
  .modal-header h2 { margin: 0; font-size: 1.125rem; }

  .close-btn {
    width: 32px; height: 32px; border-radius: 50%; background: var(--color-bg);
    border: 1px solid var(--color-border); cursor: pointer; font-size: 0.75rem;
    display: flex; align-items: center; justify-content: center;
  }

  .alert { padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); font-size: 0.875rem; }
  .alert-error { background: #FFEBEE; color: var(--color-accent); }

  .form-fields { display: flex; flex-direction: column; gap: var(--space-md); }
  .field { display: flex; flex-direction: column; gap: var(--space-xs); }
  .field label, .field-label { font-size: 0.8rem; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }

  .icon-row { display: flex; align-items: center; gap: var(--space-sm); }
  .icon-preview { width: 40px; height: 40px; border-radius: var(--radius-sm); object-fit: cover; border: 1px solid var(--color-border); }

  .radio-row { display: flex; gap: var(--space-lg); }
  .radio-label { display: flex; align-items: center; gap: var(--space-xs); cursor: pointer; font-size: 0.875rem; }
  .radio-label input { border: none; padding: 0; width: auto; margin: 0; }

  .modal-actions { display: flex; justify-content: flex-end; gap: var(--space-sm); margin-top: var(--space-lg); }
</style>
