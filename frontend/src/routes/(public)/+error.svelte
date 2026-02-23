<script lang="ts">
  import { page } from '$app/stores';
</script>

<svelte:head>
  <title>{$page.status === 404 ? 'Not found' : 'Error'}</title>
</svelte:head>

<div class="error-page">
  {#if $page.status === 404}
    <div class="error-icon">🐾</div>
    <h1 class="error-title">Page not found</h1>
    <p class="error-message">
      We couldn't find what you were looking for. It may have been moved or removed.
    </p>
  {:else}
    <div class="error-icon">⚠️</div>
    <h1 class="error-title">Something went wrong</h1>
    <p class="error-message">
      {$page.error?.message || 'An unexpected error occurred. Please try again later.'}
    </p>
  {/if}

  <div class="error-actions">
    <a href="/" class="btn btn-primary">Go home</a>
    <a href="/articles" class="btn btn-ghost">Browse articles</a>
  </div>
</div>

<style>
  .error-page {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-2xl) var(--space-lg);
    min-height: 40vh;
  }

  .error-icon {
    font-size: 4rem;
    margin-bottom: var(--space-lg);
  }

  .error-title {
    font-size: 2rem;
    margin-bottom: var(--space-md);
    color: var(--color-text);
  }

  .error-message {
    color: var(--color-text-muted);
    font-size: 1rem;
    line-height: 1.6;
    max-width: 480px;
    margin-bottom: var(--space-xl);
  }

  .error-actions {
    display: flex;
    gap: var(--space-md);
    flex-wrap: wrap;
    justify-content: center;
  }
</style>
