<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.page.title}</title>
</svelte:head>

<article class="page-view">
  <header class="page-header">
    <h1>{data.page.title}</h1>
    <div class="page-meta">
      <time datetime={data.page.updated_at}>
        Last updated {formatDate(data.page.updated_at)}
      </time>
    </div>
  </header>

  <div class="prose">
    {@html data.page.content}
  </div>
</article>

<style>
  .page-view {
    max-width: 100%;
  }

  .page-header {
    margin-bottom: var(--space-xl);
    padding-bottom: var(--space-lg);
    border-bottom: 1px solid var(--color-border);
  }

  .page-header h1 {
    margin-bottom: var(--space-sm);
  }

  .page-meta {
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }
</style>
