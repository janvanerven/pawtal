<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import type { SearchResult } from '$lib/api/types';
  import { fade, fly } from 'svelte/transition';

  interface Props {
    open: boolean;
  }

  let { open = $bindable(false) }: Props = $props();

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let loading = $state(false);
  let selectedIndex = $state(0);
  let inputEl: HTMLInputElement;
  let debounceTimer: ReturnType<typeof setTimeout>;

  $effect(() => {
    if (open) {
      setTimeout(() => inputEl?.focus(), 50);
    } else {
      query = '';
      results = [];
      selectedIndex = 0;
    }
  });

  function handleInput() {
    clearTimeout(debounceTimer);
    if (!query.trim()) {
      results = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      loading = true;
      try {
        results = await api.search(query);
        selectedIndex = 0;
      } catch {
        results = [];
      } finally {
        loading = false;
      }
    }, 250);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter' && results[selectedIndex]) {
      e.preventDefault();
      navigateTo(results[selectedIndex]);
    }
  }

  function navigateTo(result: SearchResult) {
    open = false;
    if (result.result_type === 'article') {
      goto(`/articles/${result.slug}`);
    } else if (result.result_type === 'page') {
      goto(`/${result.slug}`);
    } else if (result.result_type === 'app') {
      goto('/apps');
    }
  }

  function highlightMatch(text: string): string {
    if (!query.trim()) return text;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escaped})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }

  function typeLabel(type: string): string {
    switch (type) {
      case 'article': return 'Article';
      case 'page': return 'Page';
      case 'app': return 'App';
      default: return type;
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="search-overlay" transition:fade={{ duration: 150 }} onclick={() => open = false} onkeydown={handleKeydown}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="search-modal" transition:fly={{ y: -20, duration: 200 }} onclick={(e) => e.stopPropagation()}>
      <div class="search-input-wrapper">
        <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <input
          bind:this={inputEl}
          type="text"
          class="search-input"
          placeholder="Search articles, projects, pages..."
          bind:value={query}
          oninput={handleInput}
        />
        <kbd class="search-kbd">Esc</kbd>
      </div>

      {#if results.length > 0}
        <div class="search-results">
          {#each results as result, i (result.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <button
              class="search-result"
              class:selected={i === selectedIndex}
              onclick={() => navigateTo(result)}
              onmouseenter={() => selectedIndex = i}
            >
              <div class="result-info">
                <span class="result-type">{typeLabel(result.result_type)}</span>
                <span class="result-title">{@html highlightMatch(result.title)}</span>
              </div>
              {#if result.snippet}
                <p class="result-snippet">{@html highlightMatch(result.snippet)}</p>
              {/if}
            </button>
          {/each}
        </div>
      {:else if query.trim() && !loading}
        <div class="search-empty">
          <p>No results for "{query}"</p>
        </div>
      {/if}

      {#if loading}
        <div class="search-loading">
          <p>Searching...</p>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .search-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 200;
  }

  .search-modal {
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    width: 90%;
    max-width: 600px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--color-border);
  }

  .search-icon {
    flex-shrink: 0;
    color: var(--color-text-muted);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 1.1rem;
    color: var(--color-text);
    padding: var(--space-xs) 0;
  }

  .search-input:focus {
    outline: none;
    box-shadow: none;
  }

  .search-kbd {
    font-family: var(--font-body);
    font-size: 0.7rem;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
  }

  .search-results {
    max-height: 400px;
    overflow-y: auto;
    padding: var(--space-xs);
  }

  .search-result {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--space-md);
    border-radius: var(--radius-sm);
    cursor: pointer;
    background: transparent;
    border: none;
    color: var(--color-text);
    transition: background var(--transition-fast);
  }

  .search-result.selected,
  .search-result:hover {
    background: var(--color-bg);
  }

  .result-info {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .result-type {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-primary);
    padding: 1px 6px;
    background: rgba(232, 146, 74, 0.1);
    border-radius: var(--radius-full);
  }

  .result-title {
    font-weight: 500;
    font-size: 0.95rem;
  }

  .result-snippet {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    margin-top: var(--space-xs);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  :global(.search-result mark) {
    background: rgba(232, 146, 74, 0.2);
    color: inherit;
    border-radius: 2px;
    padding: 0 1px;
  }

  .search-empty,
  .search-loading {
    padding: var(--space-xl);
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }
</style>
