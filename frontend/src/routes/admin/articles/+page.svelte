<script lang="ts">
  import { api } from '$lib/api';
  import type { Article } from '$lib/api/types';
  import { relativeTime } from '$lib/utils';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  type StatusFilter = 'all' | 'draft' | 'published' | 'scheduled';

  let currentStatus = $derived(($page.url.searchParams.get('status') as StatusFilter) ?? 'all');
  let currentPage = $derived(Number($page.url.searchParams.get('p') ?? '1'));

  let articlesData = $state<{ data: Article[]; total: number; page: number; per_page: number } | null>(null);
  let loading = $state(false);
  let error = $state('');

  const statusTabs: { value: StatusFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'draft', label: 'Draft' },
    { value: 'published', label: 'Published' },
    { value: 'scheduled', label: 'Scheduled' },
  ];

  async function loadArticles() {
    loading = true;
    error = '';
    try {
      const statusParam = currentStatus === 'all' ? undefined : currentStatus;
      articlesData = await api.admin.listArticles(currentPage, statusParam);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load articles';
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void loadArticles();
  });

  function setStatus(status: StatusFilter) {
    const url = new URL(window.location.href);
    if (status === 'all') url.searchParams.delete('status');
    else url.searchParams.set('status', status);
    url.searchParams.delete('p');
    goto(url.toString());
  }

  function setPageNum(p: number) {
    const url = new URL(window.location.href);
    url.searchParams.set('p', String(p));
    goto(url.toString());
  }

  let totalPages = $derived(
    articlesData ? Math.ceil(articlesData.total / articlesData.per_page) : 0
  );
</script>

<svelte:head>
  <title>Articles — Pawtal CMS</title>
</svelte:head>

<div class="list-page">
  <div class="page-header">
    <h1>Articles</h1>
    <a href="/admin/articles/new" class="btn btn-primary">+ New Article</a>
  </div>

  <div class="status-tabs">
    {#each statusTabs as tab}
      <button
        class="tab-btn"
        class:active={currentStatus === tab.value}
        onclick={() => setStatus(tab.value)}
      >{tab.label}</button>
    {/each}
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if loading}
    <div class="loading-state card">Loading articles...</div>
  {:else if articlesData && articlesData.data.length > 0}
    <div class="card table-card">
      <table class="data-table">
        <thead>
          <tr>
            <th>Title</th>
            <th>Status</th>
            <th>Updated</th>
          </tr>
        </thead>
        <tbody>
          {#each articlesData.data as a (a.id)}
            <tr>
              <td>
                <a href="/admin/articles/{a.id}" class="item-title">{a.title}</a>
                {#if a.short_text}
                  <span class="item-excerpt">{a.short_text.slice(0, 80)}{a.short_text.length > 80 ? '…' : ''}</span>
                {/if}
              </td>
              <td>
                <span class="badge badge-{a.status}">{a.status}</span>
              </td>
              <td class="time-cell">{relativeTime(a.updated_at)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    {#if totalPages > 1}
      <div class="pagination">
        <button
          class="btn btn-ghost"
          disabled={currentPage <= 1}
          onclick={() => setPageNum(currentPage - 1)}
        >Previous</button>
        <span class="page-info">Page {currentPage} of {totalPages}</span>
        <button
          class="btn btn-ghost"
          disabled={currentPage >= totalPages}
          onclick={() => setPageNum(currentPage + 1)}
        >Next</button>
      </div>
    {/if}
  {:else}
    <div class="card empty-state">
      <p>No articles found.</p>
      <a href="/admin/articles/new" class="btn btn-primary" style="margin-top: var(--space-md)">Write your first article</a>
    </div>
  {/if}
</div>

<style>
  .list-page { max-width: 900px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-lg); }
  .status-tabs { display: flex; gap: 2px; margin-bottom: var(--space-lg); border-bottom: 2px solid var(--color-border); }
  .tab-btn {
    padding: var(--space-sm) var(--space-md);
    background: none; border: none; border-bottom: 2px solid transparent; margin-bottom: -2px;
    font-size: 0.875rem; font-weight: 500; color: var(--color-text-muted); cursor: pointer;
    transition: all var(--transition-fast);
  }
  .tab-btn:hover { color: var(--color-text); }
  .tab-btn.active { color: var(--color-primary); border-bottom-color: var(--color-primary); }
  .table-card { padding: 0; overflow: hidden; }
  .data-table { width: 100%; border-collapse: collapse; }
  .data-table th {
    padding: var(--space-sm) var(--space-md); text-align: left; font-size: 0.75rem; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted);
    background: var(--color-bg); border-bottom: 1px solid var(--color-border);
  }
  .data-table td { padding: var(--space-sm) var(--space-md); border-bottom: 1px solid var(--color-border); vertical-align: middle; }
  .data-table tr:last-child td { border-bottom: none; }
  .data-table tr:hover td { background: var(--color-bg); }
  .item-title { display: block; font-weight: 600; color: var(--color-text); text-decoration: none; }
  .item-title:hover { color: var(--color-primary); }
  .item-excerpt { display: block; font-size: 0.75rem; color: var(--color-text-muted); margin-top: 2px; }
  .time-cell { font-size: 0.8rem; color: var(--color-text-muted); white-space: nowrap; }
  .pagination { display: flex; align-items: center; justify-content: center; gap: var(--space-md); margin-top: var(--space-lg); }
  .page-info { font-size: 0.875rem; color: var(--color-text-muted); }
  .error-banner { background: #FFEBEE; color: var(--color-accent); padding: var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); }
  .loading-state { color: var(--color-text-muted); text-align: center; }
  .empty-state { text-align: center; color: var(--color-text-muted); padding: var(--space-2xl); }
</style>
