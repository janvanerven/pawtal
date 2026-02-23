<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';

  let { data }: { data: PageData } = $props();

  // Pagination helpers
  const currentPage = $derived(
    data.type === 'articles' ? (data.articles?.page ?? 1) :
    data.type === 'apps' ? (data.apps?.page ?? 1) : 1
  );

  const totalPages = $derived(() => {
    if (data.type === 'articles' && data.articles) {
      return Math.ceil(data.articles.total / data.articles.per_page);
    }
    if (data.type === 'apps' && data.apps) {
      return Math.ceil(data.apps.total / data.apps.per_page);
    }
    return 1;
  });
</script>

{#if data.type === 'page' && data.page}
  <article class="page-content">
    <h1>{data.page.title}</h1>
    <div class="prose">
      {@html data.page.content}
    </div>
  </article>

{:else if data.type === 'apps' && data.apps}
  <section>
    {#if data.settings?.app_catalogue_intro}
      <div class="section-intro prose">
        {@html data.settings.app_catalogue_intro}
      </div>
    {/if}
    <h1 class="section-title">App Catalogue</h1>

    {#if data.apps.data.length === 0}
      <p class="empty-state">No apps available yet.</p>
    {:else}
      <div class="apps-grid">
        {#each data.apps.data as app (app.id)}
          <div class="app-card card">
            {#if app.icon_id}
              <img
                src="/uploads/{app.icon_id}"
                alt={app.name}
                class="app-icon"
                loading="lazy"
              />
            {:else}
              <div class="app-icon-placeholder">⚡</div>
            {/if}
            <div class="app-info">
              <h3 class="app-name">{app.name}</h3>
              {#if app.description}
                <p class="app-description">{app.description}</p>
              {/if}
            </div>
            {#if app.url}
              <a href={app.url} target="_blank" rel="noopener noreferrer" class="app-link btn btn-primary">
                Visit
              </a>
            {:else if app.page_id}
              <a href="/{app.page_id}" class="app-link btn btn-primary">Learn more</a>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#if totalPages() > 1}
      <div class="pagination">
        {#if currentPage > 1}
          <a href="?page={currentPage - 1}" class="btn btn-ghost">Previous</a>
        {/if}
        <span class="page-info">Page {currentPage} of {totalPages()}</span>
        {#if currentPage < totalPages()}
          <a href="?page={currentPage + 1}" class="btn btn-ghost">Next</a>
        {/if}
      </div>
    {/if}
  </section>

{:else}
  <!-- Default: articles list -->
  <section>
    <h1 class="section-title">Latest Articles</h1>

    {#if !data.articles || data.articles.data.length === 0}
      <p class="empty-state">No articles published yet. Check back soon.</p>
    {:else}
      <div class="articles-list">
        {#each data.articles.data as article (article.id)}
          <a href="/articles/{article.slug}" class="article-card card">
            <div class="article-meta">
              <time datetime={article.publish_at ?? article.created_at}>
                {formatDate(article.publish_at ?? article.created_at)}
              </time>
            </div>
            <h2 class="article-title">{article.title}</h2>
            {#if article.short_text}
              <p class="article-excerpt">{article.short_text}</p>
            {/if}
            <span class="read-more">Read more &rarr;</span>
          </a>
        {/each}
      </div>

      {#if totalPages() > 1}
        <div class="pagination">
          {#if currentPage > 1}
            <a href="?page={currentPage - 1}" class="btn btn-ghost">Previous</a>
          {/if}
          <span class="page-info">Page {currentPage} of {totalPages()}</span>
          {#if currentPage < totalPages()}
            <a href="?page={currentPage + 1}" class="btn btn-ghost">Next</a>
          {/if}
        </div>
      {/if}
    {/if}
  </section>
{/if}

<style>
  .section-title {
    margin-bottom: var(--space-xl);
    color: var(--color-text);
  }

  .section-intro {
    margin-bottom: var(--space-xl);
    padding: var(--space-lg);
    background: var(--color-surface);
    border-radius: var(--radius-md);
    border-left: 4px solid var(--color-primary);
  }

  /* Page content */
  .page-content h1 {
    margin-bottom: var(--space-xl);
  }

  /* Article cards */
  .articles-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .article-card {
    display: block;
    text-decoration: none;
    color: var(--color-text);
    transition: box-shadow var(--transition-fast), transform var(--transition-fast);
  }

  .article-card:hover {
    box-shadow: var(--shadow-md);
    transform: translateY(-2px);
    color: var(--color-text);
  }

  .article-meta {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    margin-bottom: var(--space-sm);
  }

  .article-title {
    font-size: 1.25rem;
    margin-bottom: var(--space-sm);
    color: var(--color-text);
  }

  .article-excerpt {
    color: var(--color-text-muted);
    font-size: 0.925rem;
    line-height: 1.6;
    margin-bottom: var(--space-md);
    /* Limit to 3 lines */
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .read-more {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  /* Apps grid */
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

  .app-link {
    width: 100%;
    justify-content: center;
  }

  /* Pagination */
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

  /* Empty state */
  .empty-state {
    text-align: center;
    color: var(--color-text-muted);
    padding: var(--space-2xl) 0;
    font-size: 1rem;
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
