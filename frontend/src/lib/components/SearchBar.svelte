<script lang="ts">
  import { goto } from '$app/navigation';

  let { placeholder = 'Search...', compact = false }: { placeholder?: string; compact?: boolean } = $props();

  let query = $state('');

  function handleSubmit(e: Event) {
    e.preventDefault();
    const trimmed = query.trim();
    if (trimmed) {
      goto(`/search?q=${encodeURIComponent(trimmed)}`);
    }
  }
</script>

<form class="search-bar" class:compact onsubmit={handleSubmit} role="search">
  <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="11" cy="11" r="8"/>
    <line x1="21" y1="21" x2="16.65" y2="16.65"/>
  </svg>
  <input
    type="search"
    bind:value={query}
    {placeholder}
    aria-label="Search"
  />
</form>

<style>
  .search-bar {
    display: flex;
    align-items: center;
    position: relative;
    width: 100%;
    max-width: 320px;
  }

  .search-bar.compact {
    max-width: 200px;
  }

  .search-icon {
    position: absolute;
    left: var(--space-sm);
    color: var(--color-text-muted);
    pointer-events: none;
    flex-shrink: 0;
  }

  input {
    width: 100%;
    padding-left: calc(var(--space-sm) + 16px + var(--space-xs));
    padding-right: var(--space-sm);
    border-radius: var(--radius-full);
    font-size: 0.875rem;
    height: 36px;
    background: var(--color-bg);
    border-color: var(--color-border);
  }

  input:focus {
    background: var(--color-surface);
  }

  /* Override the global search cancel button in WebKit */
  input[type="search"]::-webkit-search-cancel-button {
    -webkit-appearance: none;
  }
</style>
