<script lang="ts">
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { onNavigate } from '$app/navigation';
  import Navigation from '$lib/components/Navigation.svelte';
  import DarkModeToggle from '$lib/components/DarkModeToggle.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import SearchModal from '$lib/components/SearchModal.svelte';

  let { data, children }: { data: LayoutData; children: import('svelte').Snippet } = $props();

  const siteTitle = $derived(data.settings?.site_title || 'Pawtal');
  const defaultTheme = $derived(data.settings?.default_theme || 'light');

  let menuOpen = $state(false);
  let searchOpen = $state(false);

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  // Close mobile menu on navigation
  $effect(() => {
    $page.url.pathname;
    menuOpen = false;
  });

  // View transitions API
  onNavigate((navigation) => {
    if (!document.startViewTransition) return;
    return new Promise((resolve) => {
      document.startViewTransition(async () => {
        resolve();
        await navigation.complete;
      });
    });
  });

  // Cmd+K search shortcut
  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      searchOpen = !searchOpen;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="public-shell">
  <!-- Header -->
  <header class="site-header">
    <div class="header-inner">
      <!-- Logo / site title -->
      <a href="/" class="site-logo">
        <span class="logo-title">{siteTitle}</span>
      </a>

      <!-- Desktop navigation in header -->
      <div class="header-nav desktop-only">
        <Navigation items={data.mainMenu.items} orientation="horizontal" />
      </div>

      <div class="header-actions">
        <button class="search-trigger desktop-only" onclick={() => searchOpen = true} title="Search (Cmd+K)">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <span class="search-hint">Cmd+K</span>
        </button>
        <DarkModeToggle defaultTheme={defaultTheme} />
        <!-- Hamburger for mobile -->
        <button
          class="hamburger mobile-only"
          onclick={toggleMenu}
          aria-label={menuOpen ? 'Close menu' : 'Open menu'}
          aria-expanded={menuOpen}
        >
          {#if menuOpen}
            <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="3" y1="6" x2="21" y2="6"/>
              <line x1="3" y1="12" x2="21" y2="12"/>
              <line x1="3" y1="18" x2="21" y2="18"/>
            </svg>
          {/if}
        </button>
      </div>
    </div>

    <!-- Mobile slide-down menu -->
    {#if menuOpen}
      <div class="mobile-menu">
        <Navigation items={data.mainMenu.items} />
      </div>
    {/if}
  </header>

  <!-- Main content area (no sidebar) -->
  <main class="site-main">
    {@render children()}
  </main>

  <!-- Footer -->
  <footer class="site-footer">
    <div class="footer-inner">
      {#if data.footerMenu.items.length > 0}
        <nav class="footer-nav" aria-label="Footer navigation">
          <Navigation items={data.footerMenu.items} orientation="horizontal" />
        </nav>
      {/if}
      <div class="footer-bottom">
        <a href="/feed.xml" class="footer-rss" title="RSS Feed">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M4 11a9 9 0 0 1 9 9"/>
            <path d="M4 4a16 16 0 0 1 16 16"/>
            <circle cx="5" cy="19" r="1"/>
          </svg>
          RSS
        </a>
        <p class="footer-credit">
          Powered by <a href="https://github.com" target="_blank" rel="noopener noreferrer">Pawtal</a>
        </p>
      </div>
    </div>
  </footer>
</div>

<SearchModal bind:open={searchOpen} />
<Toast />

<style>
  .public-shell {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  /* ---- Header ---- */
  .site-header {
    position: sticky;
    top: 0;
    z-index: 50;
    border-bottom: 1px solid var(--color-border);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    background: rgba(255, 248, 240, 0.85);
    transition: background-color var(--transition-normal);
  }

  :global([data-theme="dark"]) .site-header {
    background: rgba(12, 10, 9, 0.85);
  }

  .header-inner {
    max-width: var(--width-wide);
    margin: 0 auto;
    padding: 0 var(--space-lg);
    height: 64px;
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .site-logo {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    text-decoration: none;
    flex-shrink: 0;
  }

  .logo-title {
    font-family: var(--font-heading);
    font-size: 1.3rem;
    font-weight: 700;
    background: var(--gradient-accent);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .header-nav {
    flex: 1;
    overflow: hidden;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
    margin-left: auto;
  }

  .search-trigger {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 6px 12px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    font-size: 0.8rem;
    transition: all var(--transition-fast);
    cursor: pointer;
    background: transparent;
  }

  .search-trigger:hover {
    border-color: var(--color-primary);
    color: var(--color-text);
  }

  .search-hint {
    font-family: var(--font-body);
    font-size: 0.7rem;
    opacity: 0.6;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
  }

  .hamburger {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-sm);
    color: var(--color-text);
    transition: background var(--transition-fast);
  }

  .hamburger:hover {
    background: var(--color-bg);
  }

  .mobile-menu {
    border-top: 1px solid var(--color-border);
    padding: var(--space-md);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  /* ---- Main content ---- */
  .site-main {
    flex: 1;
    width: 100%;
    max-width: var(--width-wide);
    margin: 0 auto;
    padding: var(--space-xl) var(--space-lg);
  }

  /* ---- Footer ---- */
  .site-footer {
    background: var(--color-surface-elevated);
    border-top: 1px solid var(--color-border);
    margin-top: var(--space-3xl);
  }

  .footer-inner {
    max-width: var(--width-wide);
    margin: 0 auto;
    padding: var(--space-2xl) var(--space-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-lg);
  }

  .footer-nav :global(.nav-list) {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: center;
  }

  .footer-bottom {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
  }

  .footer-rss {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: 0.8rem;
    color: var(--color-text-muted);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  .footer-rss:hover {
    color: var(--color-primary);
  }

  .footer-credit {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  /* ---- Responsive helpers ---- */
  .desktop-only {
    display: flex;
  }

  .mobile-only {
    display: none;
  }

  @media (max-width: 768px) {
    .desktop-only {
      display: none;
    }

    .mobile-only {
      display: flex;
    }

    .site-main {
      padding: var(--space-md);
    }

    .header-inner {
      padding: 0 var(--space-md);
    }
  }

  /* ---- View Transitions ---- */
  @keyframes fade-in {
    from { opacity: 0; }
  }
  @keyframes fade-out {
    to { opacity: 0; }
  }
  @keyframes slide-from-right {
    from { transform: translateX(20px); }
  }
  @keyframes slide-to-left {
    to { transform: translateX(-20px); }
  }

  :global(::view-transition-old(root)) {
    animation: 150ms ease-out both fade-out;
  }
  :global(::view-transition-new(root)) {
    animation: 200ms ease-out both fade-in, 200ms ease-out both slide-from-right;
  }
</style>
