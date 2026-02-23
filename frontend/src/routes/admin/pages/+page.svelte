<script lang="ts">
  import { api } from '$lib/api';
  import type { Page } from '$lib/api/types';
  import { relativeTime } from '$lib/utils';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  type StatusFilter = 'all' | 'draft' | 'published' | 'scheduled';

  let currentStatus = $derived(($page.url.searchParams.get('status') as StatusFilter) ?? 'all');
  let currentPage = $derived(Number($page.url.searchParams.get('p') ?? '1'));

  let pagesData = $state<{ data: Page[]; total: number; page: number; per_page: number } | null>(null);
  let loading = $state(false);
  let error = $state('');

  const statusTabs: { value: StatusFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'draft', label: 'Draft' },
    { value: 'published', label: 'Published' },
    { value: 'scheduled', label: 'Scheduled' },
  ];

  async function loadPages() {
    loading = true;
    error = '';
    try {
      const statusParam = currentStatus === 'all' ? undefined : currentStatus;
      pagesData = await api.admin.listPages(currentPage, statusParam);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load pages';
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void loadPages();
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
    pagesData ? Math.ceil(pagesData.total / pagesData.per_page) : 0
  );
</script>

<svelte:head>
  <title>Pages — Pawtal CMS</title>
</svelte:head>

<div class="list-page">
  <div class="page-header">
    <h1>Pages</h1>
    <a href="/admin/pages/new" class="btn btn-primary">+ New Page</a>
  </div>

  <!-- Status filter tabs -->
  <div class="status-tabs">
    {#each statusTabs as tab}
      <button
        class="tab-btn"
        class:active={currentStatus === tab.value}
        onclick={() => setStatus(tab.value)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if loading}
    <div class="loading-state card">Loading pages...</div>
  {:else if pagesData && pagesData.data.length > 0}
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
          {#each pagesData.data as p (p.id)}
            <tr>
              <td>
                <a href="/admin/pages/{p.id}" class="item-title">{p.title}</a>
                <span class="item-slug">/{p.slug}</span>
              </td>
              <td>
                <span class="badge badge-{p.status}">{p.status}</span>
              </td>
              <td class="time-cell">{relativeTime(p.updated_at)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    {#if totalPages > 1}
      <div class="pagination">
        <button
          class="btn btn-ghost"
          disabled={currentPage <= 1}
          onclick={() => setPageNum(currentPage - 1)}
        >
          Previous
        </button>
        <span class="page-info">Page {currentPage} of {totalPages}</span>
        <button
          class="btn btn-ghost"
          disabled={currentPage >= totalPages}
          onclick={() => setPageNum(currentPage + 1)}
        >
          Next
        </button>
      </div>
    {/if}
  {:else}
    <div class="card empty-state">
      <p>No pages found.</p>
      <a href="/admin/pages/new" class="btn btn-primary" style="margin-top: var(--space-md)">Create your first page</a>
    </div>
  {/if}
</div>

<style>
  .list-page {
    max-width: 900px;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-lg);
  }

  .status-tabs {
    display: flex;
    gap: 2px;
    margin-bottom: var(--space-lg);
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

  .tab-btn:hover { color: var(--color-text); }

  .tab-btn.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .table-card {
    padding: 0;
    overflow: hidden;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
  }

  .data-table th {
    padding: var(--space-sm) var(--space-md);
    text-align: left;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
  }

  .data-table td {
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--color-border);
    vertical-align: middle;
  }

  .data-table tr:last-child td {
    border-bottom: none;
  }

  .data-table tr:hover td {
    background: var(--color-bg);
  }

  .item-title {
    display: block;
    font-weight: 600;
    color: var(--color-text);
    text-decoration: none;
  }

  .item-title:hover { color: var(--color-primary); }

  .item-slug {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .time-cell {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    margin-top: var(--space-lg);
  }

  .page-info {
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  .error-banner {
    background: #FFEBEE;
    color: var(--color-accent);
    padding: var(--space-md);
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-md);
  }

  .loading-state {
    color: var(--color-text-muted);
    text-align: center;
  }

  .empty-state {
    text-align: center;
    color: var(--color-text-muted);
    padding: var(--space-2xl);
  }
</style>
