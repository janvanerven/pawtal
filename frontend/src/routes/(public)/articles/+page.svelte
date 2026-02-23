<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';

  let { data }: { data: PageData } = $props();

  const totalPages = $derived(Math.ceil(data.articles.total / data.articles.per_page));
  const currentPage = $derived(data.articles.page);
</script>

<svelte:head>
  <title>Articles</title>
</svelte:head>

<section>
  <h1 class="section-title">Articles</h1>

  {#if data.articles.data.length === 0}
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
    margin-bottom: var(--space-xl);
  }

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
  }

  .article-excerpt {
    color: var(--color-text-muted);
    font-size: 0.925rem;
    line-height: 1.6;
    margin-bottom: var(--space-md);
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
</style>
