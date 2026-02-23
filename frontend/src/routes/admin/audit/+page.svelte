<script lang="ts">
  import { api } from '$lib/api';
  import type { AuditLogEntry } from '$lib/api/types';
  import { relativeTime, formatDate } from '$lib/utils';
  import { onMount } from 'svelte';

  let entries = $state<AuditLogEntry[]>([]);
  let total = $state(0);
  let currentPage = $state(1);
  let perPage = $state(20);
  let loading = $state(false);
  let error = $state('');

  async function loadAudit(p = 1) {
    loading = true;
    error = '';
    try {
      const res = await api.admin.listAuditLog(p);
      entries = res.data;
      total = res.total;
      perPage = res.per_page;
      currentPage = p;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load audit log';
    } finally {
      loading = false;
    }
  }

  onMount(() => loadAudit());

  let totalPages = $derived(Math.ceil(total / perPage));

  function actionColor(action: string): string {
    if (action.includes('create')) return '#E8F5E9';
    if (action.includes('delete') || action.includes('trash')) return '#FFEBEE';
    if (action.includes('publish')) return '#FFF3E0';
    return 'var(--color-bg)';
  }

  function actionTextColor(action: string): string {
    if (action.includes('create')) return '#2E7D32';
    if (action.includes('delete') || action.includes('trash')) return '#C62828';
    if (action.includes('publish')) return '#E65100';
    return 'var(--color-text-muted)';
  }
</script>

<svelte:head>
  <title>Audit Log — Pawtal CMS</title>
</svelte:head>

<div class="audit-page">
  <div class="page-header">
    <h1>Audit Log</h1>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if loading}
    <p class="muted-text">Loading...</p>
  {:else if entries.length === 0}
    <div class="card empty-state">No audit log entries.</div>
  {:else}
    <div class="card table-card">
      <table class="audit-table">
        <thead>
          <tr>
            <th>Time</th>
            <th>Action</th>
            <th>Type</th>
            <th>Entity</th>
            <th>Details</th>
          </tr>
        </thead>
        <tbody>
          {#each entries as entry (entry.id)}
            <tr>
              <td class="time-cell" title={formatDate(entry.created_at)}>
                {relativeTime(entry.created_at)}
              </td>
              <td>
                <span
                  class="action-badge"
                  style="background: {actionColor(entry.action)}; color: {actionTextColor(entry.action)}"
                >
                  {entry.action}
                </span>
              </td>
              <td class="type-cell">{entry.entity_type}</td>
              <td class="entity-cell">{entry.entity_id}</td>
              <td class="details-cell">{entry.details}</td>
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
          onclick={() => loadAudit(currentPage - 1)}
        >Previous</button>
        <span class="page-info">Page {currentPage} of {totalPages}</span>
        <button
          class="btn btn-ghost"
          disabled={currentPage >= totalPages}
          onclick={() => loadAudit(currentPage + 1)}
        >Next</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .audit-page { max-width: 1100px; }
  .page-header { margin-bottom: var(--space-lg); }

  .error-banner { background: #FFEBEE; color: var(--color-accent); padding: var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); }
  .muted-text { color: var(--color-text-muted); }
  .empty-state { text-align: center; color: var(--color-text-muted); padding: var(--space-2xl); }

  .table-card { padding: 0; overflow: hidden; overflow-x: auto; }

  .audit-table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
  .audit-table th {
    padding: var(--space-sm) var(--space-md); text-align: left; font-size: 0.75rem; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.05em; color: var(--color-text-muted);
    background: var(--color-bg); border-bottom: 1px solid var(--color-border); white-space: nowrap;
  }
  .audit-table td {
    padding: var(--space-sm) var(--space-md); border-bottom: 1px solid var(--color-border); vertical-align: middle;
  }
  .audit-table tr:last-child td { border-bottom: none; }
  .audit-table tr:hover td { background: var(--color-bg); }

  .time-cell { white-space: nowrap; color: var(--color-text-muted); }
  .type-cell { white-space: nowrap; color: var(--color-text-muted); font-size: 0.8rem; }
  .entity-cell { font-family: monospace; font-size: 0.75rem; color: var(--color-text-muted); max-width: 140px; overflow: hidden; text-overflow: ellipsis; }
  .details-cell { color: var(--color-text-muted); font-size: 0.8rem; max-width: 300px; }

  .action-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    font-size: 0.75rem;
    font-weight: 600;
    white-space: nowrap;
  }

  .pagination { display: flex; align-items: center; justify-content: center; gap: var(--space-md); margin-top: var(--space-lg); }
  .page-info { font-size: 0.875rem; color: var(--color-text-muted); }
</style>
