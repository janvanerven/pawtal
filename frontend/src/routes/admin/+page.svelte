<script lang="ts">
  import type { PageData } from './$types';
  import { relativeTime } from '$lib/utils';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>Dashboard — Pawtal CMS</title>
</svelte:head>

<div class="dashboard">
  <div class="page-header">
    <h1>Welcome back, {data.user.display_name}!</h1>
    <p class="subtitle">Here's what's happening on your site.</p>
  </div>

  <!-- Stats grid -->
  <div class="stats-grid">
    <div class="stat-card card">
      <div class="stat-value">{data.pageStats.draft}</div>
      <div class="stat-label">Draft Pages</div>
      <a href="/admin/pages?status=draft" class="stat-link">View all</a>
    </div>
    <div class="stat-card card">
      <div class="stat-value">{data.pageStats.published}</div>
      <div class="stat-label">Published Pages</div>
      <a href="/admin/pages?status=published" class="stat-link">View all</a>
    </div>
    <div class="stat-card card">
      <div class="stat-value">{data.articleStats.draft}</div>
      <div class="stat-label">Draft Articles</div>
      <a href="/admin/articles?status=draft" class="stat-link">View all</a>
    </div>
    <div class="stat-card card">
      <div class="stat-value">{data.articleStats.published}</div>
      <div class="stat-label">Published Articles</div>
      <a href="/admin/articles?status=published" class="stat-link">View all</a>
    </div>
  </div>

  <!-- Quick actions -->
  <div class="section">
    <h2>Quick Actions</h2>
    <div class="quick-actions">
      <a href="/admin/pages/new" class="btn btn-primary">+ New Page</a>
      <a href="/admin/articles/new" class="btn btn-secondary">+ New Article</a>
    </div>
  </div>

  <!-- Recent audit log -->
  <div class="section">
    <div class="section-header">
      <h2>Recent Activity</h2>
      <a href="/admin/audit" class="btn btn-ghost">View all</a>
    </div>

    {#if data.recentAudit.length === 0}
      <div class="card empty-state">
        <p>No activity yet.</p>
      </div>
    {:else}
      <div class="card audit-list">
        {#each data.recentAudit as entry (entry.id)}
          <div class="audit-entry">
            <div class="audit-meta">
              <span class="audit-action">{entry.action}</span>
              <span class="audit-entity">{entry.entity_type}</span>
            </div>
            <div class="audit-time">{relativeTime(entry.created_at)}</div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .dashboard {
    max-width: 960px;
  }

  .page-header {
    margin-bottom: var(--space-xl);
  }

  .page-header h1 {
    margin-bottom: var(--space-xs);
  }

  .subtitle {
    color: var(--color-text-muted);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-md);
    margin-bottom: var(--space-xl);
  }

  .stat-card {
    text-align: center;
  }

  .stat-value {
    font-family: var(--font-heading);
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--color-primary);
    line-height: 1;
    margin-bottom: var(--space-xs);
  }

  .stat-label {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    margin-bottom: var(--space-sm);
  }

  .stat-link {
    font-size: 0.8rem;
  }

  .section {
    margin-bottom: var(--space-xl);
  }

  .section h2 {
    margin-bottom: var(--space-md);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-md);
  }

  .section-header h2 {
    margin-bottom: 0;
  }

  .quick-actions {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .audit-list {
    padding: 0;
    overflow: hidden;
  }

  .audit-entry {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-border);
  }

  .audit-entry:last-child {
    border-bottom: none;
  }

  .audit-meta {
    display: flex;
    gap: var(--space-sm);
    align-items: center;
  }

  .audit-action {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text);
  }

  .audit-entity {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    background: var(--color-bg);
    padding: 2px 8px;
    border-radius: var(--radius-full);
  }

  .audit-time {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .empty-state {
    text-align: center;
    color: var(--color-text-muted);
  }

  @media (max-width: 768px) {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 480px) {
    .stats-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
