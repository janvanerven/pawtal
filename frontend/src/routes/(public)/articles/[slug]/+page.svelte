<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.article.title}</title>
</svelte:head>

<article class="article-view">
  <header class="article-header">
    <div class="article-meta">
      <time datetime={data.article.publish_at ?? data.article.created_at}>
        {formatDate(data.article.publish_at ?? data.article.created_at)}
      </time>
    </div>
    <h1>{data.article.title}</h1>
    {#if data.article.short_text}
      <p class="article-intro">{data.article.short_text}</p>
    {/if}
  </header>

  {#if data.article.content}
    <div class="prose article-body">
      {@html data.article.content}
    </div>
  {/if}

  <footer class="article-footer">
    <a href="/articles" class="back-link">&larr; Back to articles</a>
  </footer>
</article>

<style>
  .article-view {
    max-width: 100%;
  }

  .article-header {
    margin-bottom: var(--space-xl);
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--color-border);
  }

  .article-meta {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    margin-bottom: var(--space-md);
  }

  .article-header h1 {
    margin-bottom: var(--space-md);
  }

  .article-intro {
    font-size: 1.1rem;
    color: var(--color-text-muted);
    line-height: 1.7;
    font-style: italic;
  }

  .article-body {
    margin-bottom: var(--space-2xl);
  }

  .article-footer {
    padding-top: var(--space-xl);
    border-top: 1px solid var(--color-border);
  }

  .back-link {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  .back-link:hover {
    color: var(--color-primary);
  }
</style>
