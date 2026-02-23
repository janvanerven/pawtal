<script lang="ts">
  import { api } from '$lib/api';
  import type { Page, Article } from '$lib/api/types';
  import { relativeTime } from '$lib/utils';
  import { onMount } from 'svelte';

  let trashedPages = $state<Page[]>([]);
  let trashedArticles = $state<Article[]>([]);
  let loading = $state(false);
  let error = $state('');
  let successMsg = $state('');

  async function loadTrash() {
    loading = true;
    error = '';
    try {
      const data = await api.admin.listTrash();
      trashedPages = data.pages;
      trashedArticles = data.articles;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load trash';
    } finally {
      loading = false;
    }
  }

  onMount(() => loadTrash());

  async function restorePage(page: Page) {
    try {
      await api.admin.restorePage(page.id);
      trashedPages = trashedPages.filter(p => p.id !== page.id);
      successMsg = `"${page.title}" restored.`;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Restore failed';
    }
  }

  async function restoreArticle(article: Article) {
    try {
      await api.admin.restoreArticle(article.id);
      trashedArticles = trashedArticles.filter(a => a.id !== article.id);
      successMsg = `"${article.title}" restored.`;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Restore failed';
    }
  }

  async function emptyTrash() {
    if (!confirm('Permanently delete all items older than 30 days? This cannot be undone.')) return;
    try {
      const result = await api.admin.emptyTrash();
      successMsg = `Deleted ${result.deleted_pages} pages and ${result.deleted_articles} articles.`;
      loadTrash();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Empty trash failed';
    }
  }

  // Check if an item is older than 30 days
  function isOld(dateStr: string | null): boolean {
    if (!dateStr) return false;
    const date = new Date(dateStr);
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
    return date < thirtyDaysAgo;
  }
</script>

<svelte:head>
  <title>Trash — Pawtal CMS</title>
</svelte:head>

<div class="trash-page">
  <div class="page-header">
    <h1>Trash</h1>
    <button class="btn btn-danger" onclick={emptyTrash}>Empty Trash (30+ days)</button>
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
    <!-- Trashed Pages -->
    <section class="trash-section">
      <h2>Pages ({trashedPages.length})</h2>

      {#if trashedPages.length === 0}
        <div class="card empty-msg">No trashed pages.</div>
      {:else}
        <div class="card trash-list">
          {#each trashedPages as p (p.id)}
            <div class="trash-item">
              <div class="trash-info">
                <span class="trash-title">{p.title}</span>
                <span class="trash-meta">
                  Trashed {relativeTime(p.trashed_at ?? p.updated_at)}
                  {#if isOld(p.trashed_at)}
                    <span class="old-badge">Will be auto-deleted</span>
                  {/if}
                </span>
              </div>
              <button
                type="button"
                class="btn btn-secondary"
                onclick={() => restorePage(p)}
              >Restore</button>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Trashed Articles -->
    <section class="trash-section">
      <h2>Articles ({trashedArticles.length})</h2>

      {#if trashedArticles.length === 0}
        <div class="card empty-msg">No trashed articles.</div>
      {:else}
        <div class="card trash-list">
          {#each trashedArticles as a (a.id)}
            <div class="trash-item">
              <div class="trash-info">
                <span class="trash-title">{a.title}</span>
                <span class="trash-meta">
                  Trashed {relativeTime(a.trashed_at ?? a.updated_at)}
                  {#if isOld(a.trashed_at)}
                    <span class="old-badge">Will be auto-deleted</span>
                  {/if}
                </span>
              </div>
              <button
                type="button"
                class="btn btn-secondary"
                onclick={() => restoreArticle(a)}
              >Restore</button>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</div>

<style>
  .trash-page { max-width: 800px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-lg); }

  .alert { padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); font-size: 0.875rem; }
  .alert-error { background: #FFEBEE; color: var(--color-accent); }
  .alert-success { background: #E8F5E9; color: #2E7D32; }

  .muted-text { color: var(--color-text-muted); }

  .trash-section { margin-bottom: var(--space-xl); }
  .trash-section h2 { font-size: 1rem; margin-bottom: var(--space-md); color: var(--color-text-muted); }

  .empty-msg { color: var(--color-text-muted); text-align: center; padding: var(--space-lg); }

  .trash-list { padding: 0; overflow: hidden; }

  .trash-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-border);
  }
  .trash-item:last-child { border-bottom: none; }

  .trash-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .trash-title { font-weight: 600; font-size: 0.9rem; }
  .trash-meta { font-size: 0.8rem; color: var(--color-text-muted); display: flex; align-items: center; gap: var(--space-sm); flex-wrap: wrap; }

  .old-badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 8px;
    border-radius: var(--radius-full);
    font-size: 0.7rem;
    font-weight: 600;
    background: #FFEBEE;
    color: #C62828;
  }
</style>
