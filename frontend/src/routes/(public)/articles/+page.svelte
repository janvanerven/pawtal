<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';
  import InView from '$lib/components/InView.svelte';

  let { data }: { data: PageData } = $props();

  const totalPages = $derived(Math.ceil(data.articles.total / data.articles.per_page));
  const currentPage = $derived(data.articles.page);
</script>

<svelte:head>
  <title>Articles</title>
</svelte:head>

<section>
  <div class="page-header">
    <h1>Articles</h1>
    <p class="page-subtitle">Thoughts, tutorials, and project updates.</p>
  </div>

  {#if data.articles.data.length === 0}
    <p class="empty-state">No articles published yet. Check back soon.</p>
  {:else}
    <div class="articles-grid">
      {#each data.articles.data as article, i (article.id)}
        <InView delay={i * 80}>
          <a href="/articles/{article.slug}" class="article-card">
            {#if article.cover_image_id}
              <div class="article-cover">
                <img
                  src="/uploads/{article.cover_image_id}/medium.webp"
                  alt={article.title}
                  loading="lazy"
                />
              </div>
            {/if}
            <div class="article-card-body">
              <div class="article-meta">
                <time datetime={article.publish_at ?? article.created_at}>
                  {formatDate(article.publish_at ?? article.created_at)}
                </time>
                {#if article.reading_time_minutes > 0}
                  <span class="reading-time">{article.reading_time_minutes} min read</span>
                {/if}
              </div>
              <h2 class="article-title">{article.title}</h2>
              {#if article.short_text}
                <p class="article-excerpt">{article.short_text}</p>
              {/if}
              <span class="read-more">Read more &rarr;</span>
            </div>
          </a>
        </InView>
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
  .page-header {
    margin-bottom: var(--space-2xl);
  }

  .page-header h1 {
    margin-bottom: var(--space-sm);
  }

  .page-subtitle {
    color: var(--color-text-muted);
    font-size: 1.1rem;
  }

  .articles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: var(--space-lg);
  }

  .article-card {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--color-surface);
    box-shadow: var(--shadow-sm);
    text-decoration: none;
    color: var(--color-text);
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  }

  .article-card:hover {
    transform: translateY(-4px);
    box-shadow: var(--shadow-lg);
    color: var(--color-text);
  }

  .article-cover {
    width: 100%;
    aspect-ratio: 16/9;
    overflow: hidden;
  }

  .article-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform var(--transition-normal);
  }

  .article-card:hover .article-cover img {
    transform: scale(1.03);
  }

  .article-card-body {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    flex: 1;
  }

  .article-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .reading-time::before {
    content: '\00B7';
    margin-right: var(--space-sm);
  }

  .article-title {
    font-size: 1.2rem;
    font-weight: 600;
    line-height: 1.3;
  }

  .article-excerpt {
    color: var(--color-text-muted);
    font-size: 0.875rem;
    line-height: 1.6;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    flex: 1;
  }

  .read-more {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-primary);
    margin-top: auto;
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    margin-top: var(--space-2xl);
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

  @media (max-width: 480px) {
    .articles-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
