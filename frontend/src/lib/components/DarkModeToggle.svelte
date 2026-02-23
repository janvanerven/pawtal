<script lang="ts">
  // Priority order for initial preference:
  // 1. localStorage value (user's explicit choice)
  // 2. settings default (passed as prop)
  // 3. system preference (prefers-color-scheme)

  let { defaultTheme = 'light' }: { defaultTheme?: string } = $props();

  function getInitialTheme(): boolean {
    if (typeof window === 'undefined') return false;

    const stored = localStorage.getItem('theme');
    if (stored === 'dark') return true;
    if (stored === 'light') return false;

    if (defaultTheme === 'dark') return true;

    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }

  let isDark = $state(false);

  function applyTheme(dark: boolean) {
    document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
    localStorage.setItem('theme', dark ? 'dark' : 'light');
  }

  function toggle() {
    isDark = !isDark;
    applyTheme(isDark);
  }

  // Initialize on mount (runs only in browser)
  $effect(() => {
    isDark = getInitialTheme();
    applyTheme(isDark);
  });
</script>

<button
  class="dark-mode-toggle"
  onclick={toggle}
  aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
  title={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
>
  {#if isDark}
    <!-- Sun icon -->
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="12" cy="12" r="5"/>
      <line x1="12" y1="1" x2="12" y2="3"/>
      <line x1="12" y1="21" x2="12" y2="23"/>
      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
      <line x1="1" y1="12" x2="3" y2="12"/>
      <line x1="21" y1="12" x2="23" y2="12"/>
      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
    </svg>
  {:else}
    <!-- Moon icon -->
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
    </svg>
  {/if}
</button>

<style>
  .dark-mode-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-full);
    color: var(--color-text-muted);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .dark-mode-toggle:hover {
    background: var(--color-border);
    color: var(--color-text);
  }
</style>
