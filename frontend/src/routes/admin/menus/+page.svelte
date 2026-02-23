<script lang="ts">
  import { api } from '$lib/api';
  import type { MenuItem } from '$lib/api/types';
  import { onMount } from 'svelte';

  type MenuName = 'main' | 'footer';
  type LinkType = 'page' | 'article' | 'url' | 'app_catalogue';

  interface EditableItem extends Partial<MenuItem> {
    _key: string; // local unique key for #each
    label: string;
    link_type: LinkType;
    link_target: string;
    parent_id: string | null;
    sort_order: number;
  }

  let activeMenu = $state<MenuName>('main');
  let items = $state<EditableItem[]>([]);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state('');
  let successMsg = $state('');

  // New item form
  let newLabel = $state('');
  let newLinkType = $state<LinkType>('url');
  let newLinkTarget = $state('');

  let keyCounter = 0;
  function makeKey() { return `item-${++keyCounter}`; }

  async function loadMenu(name: MenuName) {
    loading = true;
    error = '';
    try {
      const res = await api.admin.getMenu(name);
      items = res.items.map(item => ({
        ...item,
        _key: makeKey(),
      }));
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load menu';
    } finally {
      loading = false;
    }
  }

  onMount(() => loadMenu('main'));

  function switchMenu(name: MenuName) {
    activeMenu = name;
    loadMenu(name);
  }

  function addItem() {
    if (!newLabel.trim()) { error = 'Label is required.'; return; }
    if (!newLinkTarget.trim() && newLinkType !== 'app_catalogue') { error = 'Link target is required.'; return; }
    error = '';

    items = [
      ...items,
      {
        _key: makeKey(),
        label: newLabel.trim(),
        link_type: newLinkType,
        link_target: newLinkTarget.trim(),
        parent_id: null,
        sort_order: items.length,
      }
    ];

    newLabel = '';
    newLinkTarget = '';
  }

  function removeItem(key: string) {
    items = items.filter(i => i._key !== key);
  }

  function moveItemUp(index: number) {
    if (index === 0) return;
    const arr = [...items];
    [arr[index - 1], arr[index]] = [arr[index], arr[index - 1]];
    items = arr;
  }

  function moveItemDown(index: number) {
    if (index >= items.length - 1) return;
    const arr = [...items];
    [arr[index], arr[index + 1]] = [arr[index + 1], arr[index]];
    items = arr;
  }

  function indentItem(index: number) {
    if (index === 0) return;
    const arr = [...items];
    const parent = arr[index - 1];
    arr[index] = { ...arr[index], parent_id: parent.id ?? null };
    items = arr;
  }

  function outdentItem(index: number) {
    const arr = [...items];
    arr[index] = { ...arr[index], parent_id: null };
    items = arr;
  }

  async function saveMenu() {
    saving = true;
    error = '';
    successMsg = '';
    try {
      const payload = items.map((item, i) => ({
        id: item.id,
        label: item.label,
        link_type: item.link_type,
        link_target: item.link_target,
        parent_id: item.parent_id ?? null,
        sort_order: i,
      }));
      await api.admin.updateMenu(activeMenu, payload);
      successMsg = 'Menu saved.';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Save failed';
    } finally {
      saving = false;
    }
  }

  const linkTypeLabels: Record<LinkType, string> = {
    url: 'External URL',
    page: 'Page',
    article: 'Article',
    app_catalogue: 'App Catalogue',
  };
</script>

<svelte:head>
  <title>Menus — Pawtal CMS</title>
</svelte:head>

<div class="menus-page">
  <div class="page-header">
    <h1>Menu Editor</h1>
    <button class="btn btn-primary" disabled={saving} onclick={saveMenu}>
      {saving ? 'Saving...' : 'Save Menu'}
    </button>
  </div>

  <!-- Tab switcher -->
  <div class="menu-tabs">
    {#each (['main', 'footer'] as MenuName[]) as name}
      <button
        class="tab-btn"
        class:active={activeMenu === name}
        onclick={() => switchMenu(name)}
      >{name === 'main' ? 'Main Menu' : 'Footer Menu'}</button>
    {/each}
  </div>

  {#if error}
    <div class="alert alert-error">{error}</div>
  {/if}
  {#if successMsg}
    <div class="alert alert-success">{successMsg}</div>
  {/if}

  {#if loading}
    <p class="muted-text">Loading...</p>
  {:else}
    <!-- Current menu items -->
    {#if items.length === 0}
      <div class="card empty-state">No items in this menu yet.</div>
    {:else}
      <div class="menu-list card">
        {#each items as item, i (item._key)}
          <div class="menu-item" class:indented={!!item.parent_id}>
            <div class="item-indent-indicator">
              {#if item.parent_id}
                <span class="indent-icon">↳</span>
              {/if}
            </div>
            <div class="item-info">
              <span class="item-label">{item.label}</span>
              <span class="item-type badge badge-draft">{linkTypeLabels[item.link_type as LinkType]}</span>
              {#if item.link_target}
                <span class="item-target">{item.link_target}</span>
              {/if}
            </div>
            <div class="item-controls">
              <button class="ctrl-btn" title="Move up" onclick={() => moveItemUp(i)} disabled={i === 0}>↑</button>
              <button class="ctrl-btn" title="Move down" onclick={() => moveItemDown(i)} disabled={i >= items.length - 1}>↓</button>
              <button class="ctrl-btn" title="Indent" onclick={() => indentItem(i)} disabled={i === 0 || !!item.parent_id}>⇥</button>
              <button class="ctrl-btn" title="Outdent" onclick={() => outdentItem(i)} disabled={!item.parent_id}>⇤</button>
              <button class="ctrl-btn danger" title="Remove" onclick={() => removeItem(item._key)}>✕</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Add item form -->
    <div class="add-item-form card">
      <h3>Add Menu Item</h3>
      <div class="add-fields">
        <div class="field">
          <label for="item-label">Label</label>
          <input id="item-label" type="text" bind:value={newLabel} placeholder="Home" />
        </div>

        <div class="field">
          <label for="item-type">Type</label>
          <select id="item-type" bind:value={newLinkType}>
            <option value="url">External URL</option>
            <option value="page">Page</option>
            <option value="article">Article</option>
            <option value="app_catalogue">App Catalogue</option>
          </select>
        </div>

        {#if newLinkType !== 'app_catalogue'}
          <div class="field">
            <label for="item-target">
              {#if newLinkType === 'url'}URL{:else}Slug{/if}
            </label>
            <input
              id="item-target"
              type="text"
              bind:value={newLinkTarget}
              placeholder={newLinkType === 'url' ? 'https://example.com' : 'page-slug'}
            />
          </div>
        {/if}

        <button type="button" class="btn btn-secondary" onclick={addItem}>Add Item</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .menus-page { max-width: 800px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-lg); }

  .menu-tabs { display: flex; gap: 2px; margin-bottom: var(--space-lg); border-bottom: 2px solid var(--color-border); }
  .tab-btn {
    padding: var(--space-sm) var(--space-md); background: none; border: none;
    border-bottom: 2px solid transparent; margin-bottom: -2px; font-size: 0.875rem;
    font-weight: 500; color: var(--color-text-muted); cursor: pointer; transition: all var(--transition-fast);
  }
  .tab-btn:hover { color: var(--color-text); }
  .tab-btn.active { color: var(--color-primary); border-bottom-color: var(--color-primary); }

  .alert { padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); font-size: 0.875rem; }
  .alert-error { background: #FFEBEE; color: var(--color-accent); }
  .alert-success { background: #E8F5E9; color: #2E7D32; }

  .muted-text { color: var(--color-text-muted); }
  .empty-state { color: var(--color-text-muted); text-align: center; padding: var(--space-xl); }

  .menu-list { padding: 0; overflow: hidden; margin-bottom: var(--space-md); }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    transition: background var(--transition-fast);
  }
  .menu-item:last-child { border-bottom: none; }
  .menu-item:hover { background: var(--color-bg); }
  .menu-item.indented { padding-left: var(--space-2xl); }

  .item-indent-indicator { width: 20px; flex-shrink: 0; }
  .indent-icon { color: var(--color-text-muted); font-size: 0.875rem; }

  .item-info { flex: 1; display: flex; align-items: center; gap: var(--space-sm); min-width: 0; flex-wrap: wrap; }
  .item-label { font-weight: 600; font-size: 0.9rem; }
  .item-target { font-size: 0.75rem; color: var(--color-text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px; }

  .item-controls { display: flex; gap: 2px; flex-shrink: 0; }
  .ctrl-btn {
    width: 28px; height: 28px; border-radius: var(--radius-sm); background: transparent;
    border: 1px solid var(--color-border); cursor: pointer; font-size: 0.75rem;
    display: flex; align-items: center; justify-content: center; transition: all var(--transition-fast);
  }
  .ctrl-btn:hover:not(:disabled) { background: var(--color-bg); border-color: var(--color-primary); }
  .ctrl-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .ctrl-btn.danger:hover:not(:disabled) { background: #FFEBEE; border-color: var(--color-accent); color: var(--color-accent); }

  /* .add-item-form uses default card styling */
  .add-item-form h3 { font-size: 0.9rem; margin-bottom: var(--space-md); color: var(--color-text-muted); font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }

  .add-fields { display: grid; grid-template-columns: 1fr 1fr 1fr auto; gap: var(--space-md); align-items: end; }
  .field { display: flex; flex-direction: column; gap: var(--space-xs); }
  .field label { font-size: 0.75rem; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }

  @media (max-width: 700px) {
    .add-fields { grid-template-columns: 1fr; }
  }
</style>
