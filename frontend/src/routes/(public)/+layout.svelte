<script lang="ts">
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import Navigation from '$lib/components/Navigation.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import DarkModeToggle from '$lib/components/DarkModeToggle.svelte';

  let { data, children }: { data: LayoutData; children: import('svelte').Snippet } = $props();

  const siteTitle = $derived(data.settings?.site_title || 'Pawtal');
  const defaultTheme = $derived(data.settings?.default_theme || 'light');

  let menuOpen = $state(false);

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  // Close mobile menu on navigation
  $effect(() => {
    $page.url.pathname;
    menuOpen = false;
  });
</script>

<div class="public-shell">
  <!-- Header -->
  <header class="site-header">
    <div class="header-inner">
      <!-- Logo / site title -->
      <a href="/" class="site-logo">
        <span class="logo-paw">🐾</span>
        <span class="logo-title">{siteTitle}</span>
      </a>

      <!-- Desktop navigation in header -->
      <div class="header-nav desktop-only">
        <Navigation items={data.mainMenu.items} orientation="horizontal" />
      </div>

      <div class="header-actions">
        <div class="desktop-only">
          <SearchBar compact />
        </div>
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
        <div class="mobile-search">
          <SearchBar />
        </div>
        <Navigation items={data.mainMenu.items} />
      </div>
    {/if}
  </header>

  <!-- Body: sidebar + content -->
  <div class="site-body">
    <!-- Sidebar (desktop only) -->
    <aside class="site-sidebar desktop-only">
      <Navigation items={data.mainMenu.items} />
    </aside>

    <!-- Main content area -->
    <main class="site-main">
      {@render children()}
    </main>
  </div>

  <!-- Footer -->
  <footer class="site-footer">
    <div class="footer-inner">
      {#if data.footerMenu.items.length > 0}
        <nav class="footer-nav" aria-label="Footer navigation">
          <Navigation items={data.footerMenu.items} orientation="horizontal" />
        </nav>
      {/if}
      <p class="footer-credit">
        Powered by <a href="https://github.com" target="_blank" rel="noopener noreferrer">Pawtal</a>
      </p>
    </div>
  </footer>
</div>

<style>
  .public-shell {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  /* ---- Header ---- */
  .site-header {
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    box-shadow: var(--shadow-sm);
    position: sticky;
    top: 0;
    z-index: 50;
  }

  .header-inner {
    max-width: 1280px;
    margin: 0 auto;
    padding: 0 var(--space-lg);
    height: 60px;
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

  .logo-paw {
    font-size: 1.4rem;
  }

  .logo-title {
    font-family: var(--font-heading);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-primary);
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

  .hamburger {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
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

  .mobile-search {
    display: flex;
    justify-content: stretch;
  }

  .mobile-search :global(.search-bar) {
    max-width: 100%;
    width: 100%;
  }

  /* ---- Body / Sidebar ---- */
  .site-body {
    flex: 1;
    display: flex;
    max-width: 1280px;
    width: 100%;
    margin: 0 auto;
    padding: var(--space-xl) var(--space-lg);
    gap: var(--space-xl);
    align-items: flex-start;
  }

  .site-sidebar {
    width: 240px;
    flex-shrink: 0;
    position: sticky;
    top: calc(60px + var(--space-lg));
  }

  .site-main {
    flex: 1;
    min-width: 0;
    max-width: 900px;
  }

  /* ---- Footer ---- */
  .site-footer {
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
    margin-top: auto;
  }

  .footer-inner {
    max-width: 1280px;
    margin: 0 auto;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-md);
  }

  .footer-nav :global(.nav-list) {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: center;
  }

  .footer-credit {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  /* ---- Responsive helpers ---- */
  .desktop-only {
    display: block;
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

    .site-body {
      padding: var(--space-md);
    }

    .header-inner {
      padding: 0 var(--space-md);
    }
  }
</style>
