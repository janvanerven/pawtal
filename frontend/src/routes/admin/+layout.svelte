<script lang="ts">
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { api } from '$lib/api';

  let { data, children }: { data: LayoutData; children: import('svelte').Snippet } = $props();

  const navLinks = [
    { href: '/admin', label: 'Dashboard', icon: '⊞' },
    { href: '/admin/pages', label: 'Pages', icon: '📄' },
    { href: '/admin/articles', label: 'Articles', icon: '📰' },
    { href: '/admin/media', label: 'Media', icon: '🖼' },
    { href: '/admin/apps', label: 'Apps', icon: '⚡' },
    { href: '/admin/menus', label: 'Menus', icon: '☰' },
    { href: '/admin/settings', label: 'Settings', icon: '⚙' },
    { href: '/admin/trash', label: 'Trash', icon: '🗑' },
    { href: '/admin/audit', label: 'Audit Log', icon: '📋' },
  ];

  function isActive(href: string): boolean {
    if (href === '/admin') return $page.url.pathname === '/admin';
    return $page.url.pathname.startsWith(href);
  }

  async function handleLogout() {
    await fetch('/api/auth/logout', { method: 'POST' });
    window.location.href = '/';
  }
</script>

<div class="admin-shell">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <a href="/admin" class="logo">
        <span class="logo-paw">🐾</span>
        <span class="logo-text">Pawtal</span>
      </a>
    </div>

    <nav class="sidebar-nav">
      {#each navLinks as link}
        <a
          href={link.href}
          class="nav-link"
          class:active={isActive(link.href)}
        >
          <span class="nav-icon">{link.icon}</span>
          <span class="nav-label">{link.label}</span>
        </a>
      {/each}
    </nav>

    <div class="sidebar-footer">
      <div class="user-info">
        <div class="user-avatar">{data.user.display_name.charAt(0).toUpperCase()}</div>
        <div class="user-details">
          <span class="user-name">{data.user.display_name}</span>
          <span class="user-role">{data.user.role}</span>
        </div>
      </div>
      <button class="btn btn-ghost logout-btn" onclick={handleLogout}>
        Sign out
      </button>
    </div>
  </aside>

  <!-- Main content -->
  <main class="admin-main">
    {@render children()}
  </main>

  <!-- Mobile bottom nav -->
  <nav class="mobile-nav">
    {#each navLinks.slice(0, 5) as link}
      <a
        href={link.href}
        class="mobile-nav-link"
        class:active={isActive(link.href)}
      >
        <span class="nav-icon">{link.icon}</span>
        <span class="mobile-nav-label">{link.label}</span>
      </a>
    {/each}
  </nav>
</div>

<style>
  .admin-shell {
    display: flex;
    min-height: 100vh;
    background: var(--color-bg);
  }

  /* Sidebar */
  .sidebar {
    width: 260px;
    min-height: 100vh;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    position: sticky;
    top: 0;
    height: 100vh;
    overflow-y: auto;
    flex-shrink: 0;
    box-shadow: var(--shadow-sm);
    z-index: 10;
  }

  .sidebar-header {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--color-border);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    text-decoration: none;
  }

  .logo-paw {
    font-size: 1.5rem;
  }

  .logo-text {
    font-family: var(--font-heading);
    font-size: 1.375rem;
    font-weight: 700;
    color: var(--color-primary);
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--space-md) var(--space-sm);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    text-decoration: none;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .nav-link:hover {
    background: var(--color-bg);
    color: var(--color-text);
  }

  .nav-link.active {
    background: rgba(232, 146, 74, 0.12);
    color: var(--color-primary);
  }

  .nav-icon {
    font-size: 1rem;
    width: 1.25rem;
    text-align: center;
    flex-shrink: 0;
  }

  .sidebar-footer {
    padding: var(--space-md);
    border-top: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .user-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--color-primary);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.875rem;
    flex-shrink: 0;
  }

  .user-details {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .user-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-role {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: capitalize;
  }

  .logout-btn {
    width: 100%;
    justify-content: center;
    font-size: 0.8rem;
    padding: var(--space-xs) var(--space-sm);
  }

  /* Main content */
  .admin-main {
    flex: 1;
    min-width: 0;
    padding: var(--space-xl);
    overflow-y: auto;
  }

  /* Mobile nav */
  .mobile-nav {
    display: none;
  }

  @media (max-width: 768px) {
    .admin-shell {
      flex-direction: column;
    }

    .sidebar {
      display: none;
    }

    .admin-main {
      padding: var(--space-md);
      padding-bottom: 80px;
    }

    .mobile-nav {
      display: flex;
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      background: var(--color-surface);
      border-top: 1px solid var(--color-border);
      z-index: 100;
      box-shadow: 0 -2px 8px rgba(61, 50, 41, 0.08);
    }

    .mobile-nav-link {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
      padding: var(--space-sm) var(--space-xs);
      color: var(--color-text-muted);
      text-decoration: none;
      font-size: 0.65rem;
    }

    .mobile-nav-link.active {
      color: var(--color-primary);
    }

    .mobile-nav-link .nav-icon {
      font-size: 1.2rem;
    }

    .mobile-nav-label {
      font-size: 0.6rem;
    }
  }
</style>
