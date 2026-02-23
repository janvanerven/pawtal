<script lang="ts">
  import type { PageData } from './$types';
  import type { SearchResult } from '$lib/api/types';

  let { data }: { data: PageData } = $props();

  // Map result_type to a display label and URL
  function resultUrl(result: SearchResult): string {
    switch (result.result_type) {
      case 'article': return `/articles/${result.slug}`;
      case 'app':     return `/apps`;
      default:        return `/${result.slug}`;
    }
  }

  const typeLabel: Record<SearchResult['result_type'], string> = {
    page: 'Page',
    article: 'Article',
    app: 'App',
  };

  const typeBadgeClass: Record<SearchResult['result_type'], string> = {
    page: 'badge-page',
    article: 'badge-article',
    app: 'badge-app',
  };
</script>

<svelte:head>
  <title>{data.query ? `Search: ${data.query}` : 'Search'}</title>
</svelte:head>

<section class="search-page">
  <h1 class="section-title">
    {#if data.query}
      Search results for &ldquo;{data.query}&rdquo;
    {:else}
      Search
    {/if}
  </h1>

  {#if !data.query}
    <p class="hint">Enter a search term in the search bar above to find pages, articles, and apps.</p>

  {:else if data.results.length === 0}
    <div class="no-results">
      <p class="no-results-message">No results found for &ldquo;{data.query}&rdquo;.</p>
      <p class="no-results-hint">Try different keywords or check your spelling.</p>
    </div>

  {:else}
    <p class="result-count">{data.results.length} result{data.results.length !== 1 ? 's' : ''} found</p>

    <ul class="results-list">
      {#each data.results as result (result.id)}
        <li class="result-item card">
          <div class="result-header">
            <a href={resultUrl(result)} class="result-title">{result.title}</a>
            <span class="badge {typeBadgeClass[result.result_type]}">
              {typeLabel[result.result_type]}
            </span>
          </div>
          {#if result.snippet}
            <p class="result-snippet">{result.snippet}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .section-title {
    margin-bottom: var(--space-lg);
  }

  .result-count {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    margin-bottom: var(--space-lg);
  }

  .hint {
    color: var(--color-text-muted);
    padding: var(--space-2xl) 0;
    text-align: center;
  }

  .no-results {
    padding: var(--space-2xl) 0;
    text-align: center;
  }

  .no-results-message {
    font-size: 1.1rem;
    color: var(--color-text);
    margin-bottom: var(--space-sm);
  }

  .no-results-hint {
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }

  .results-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .result-item {
    padding: var(--space-lg);
  }

  .result-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-md);
    margin-bottom: var(--space-sm);
  }

  .result-title {
    font-family: var(--font-heading);
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-text);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  .result-title:hover {
    color: var(--color-primary);
  }

  .result-snippet {
    font-size: 0.9rem;
    color: var(--color-text-muted);
    line-height: 1.6;
    /* Limit to 3 lines */
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* Type badges */
  .badge-page {
    background: #E8F0FE;
    color: #1A56DB;
    flex-shrink: 0;
  }

  .badge-article {
    background: #FFF3E0;
    color: #E65100;
    flex-shrink: 0;
  }

  .badge-app {
    background: #E8F5E9;
    color: #2E7D32;
    flex-shrink: 0;
  }
</style>
