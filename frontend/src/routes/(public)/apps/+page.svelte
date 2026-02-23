<script lang="ts">
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  const totalPages = $derived(Math.ceil(data.apps.total / data.apps.per_page));
  const currentPage = $derived(data.apps.page);
</script>

<svelte:head>
  <title>App Catalogue</title>
</svelte:head>

<section>
  <h1 class="section-title">App Catalogue</h1>

  {#if data.settings?.app_catalogue_intro}
    <div class="intro prose">
      {@html data.settings.app_catalogue_intro}
    </div>
  {/if}

  {#if data.apps.data.length === 0}
    <p class="empty-state">No apps listed yet.</p>
  {:else}
    <div class="apps-grid">
      {#each data.apps.data as app (app.id)}
        <div class="app-card card">
          <div class="app-icon-wrap">
            {#if app.icon_id}
              <img
                src="/uploads/{app.icon_id}/{app.icon_filename}"
                alt={app.name}
                class="app-icon"
                loading="lazy"
              />
            {:else}
              <div class="app-icon-placeholder">⚡</div>
            {/if}
          </div>
          <div class="app-info">
            <h2 class="app-name">{app.name}</h2>
            {#if app.description}
              <p class="app-description">{app.description}</p>
            {/if}
          </div>
          <div class="app-actions">
            {#if app.url}
              <a
                href={app.url}
                target="_blank"
                rel="noopener noreferrer"
                class="btn btn-primary"
              >
                Visit
              </a>
            {:else if app.page_id}
              <a href="/{app.page_id}" class="btn btn-primary">Learn more</a>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    {#if totalPages > 1}
      <div class="pagination">
        {#if currentPage > 1}
          <a href="?page={currentPage - 1}" class="btn btn-ghost">Previous</a>
        {/if}
        <span class="page-info">Page {currentPage} of {totalPages}</span>
        {#if currentPage < totalPages}
          <a href="?page={currentPage + 1}" class="btn btn-ghost">Next</a>
        {/if}
      </div>
    {/if}
  {/if}
</section>

<style>
  .section-title {
    margin-bottom: var(--space-lg);
  }

  .intro {
    margin-bottom: var(--space-xl);
    padding: var(--space-lg);
    background: var(--color-surface);
    border-radius: var(--radius-md);
    border-left: 4px solid var(--color-primary);
  }

  .apps-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .app-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-md);
    transition: box-shadow var(--transition-fast), transform var(--transition-fast);
  }

  .app-card:hover {
    box-shadow: var(--shadow-md);
    transform: translateY(-2px);
  }

  .app-icon-wrap {
    flex-shrink: 0;
  }

  .app-icon {
    width: 72px;
    height: 72px;
    border-radius: var(--radius-md);
    object-fit: contain;
  }

  .app-icon-placeholder {
    width: 72px;
    height: 72px;
    border-radius: var(--radius-md);
    background: var(--color-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
  }

  .app-info {
    flex: 1;
    min-width: 0;
  }

  .app-name {
    font-size: 1rem;
    margin-bottom: var(--space-xs);
  }

  .app-description {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .app-actions {
    width: 100%;
  }

  .app-actions .btn {
    width: 100%;
    justify-content: center;
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    margin-top: var(--space-xl);
  }

  .page-info {
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  .empty-state {
    text-align: center;
    color: var(--color-text-muted);
    padding: var(--space-2xl) 0;
  }

  @media (max-width: 768px) {
    .apps-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 480px) {
    .apps-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
