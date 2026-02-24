<script lang="ts">
  import { page } from '$app/stores';
  import type { MenuItem } from '$lib/api/types';

  let { items, orientation = 'vertical' }: {
    items: MenuItem[];
    orientation?: 'vertical' | 'horizontal';
  } = $props();

  // Build a URL from a menu item's link_type and link_target
  function itemUrl(item: MenuItem): string {
    switch (item.link_type) {
      case 'page':        return `/${item.link_target}`;
      case 'article':     return `/articles/${item.link_target}`;
      case 'app_catalogue': return '/apps';
      case 'url':         return item.link_target;
      default:            return '#';
    }
  }

  // Check whether the given URL is the active route
  function isActive(url: string): boolean {
    const path = $page.url.pathname;
    if (url === '/') return path === '/';
    return path === url || path.startsWith(url + '/');
  }

  // Separate top-level items from children
  const rootItems = $derived(items.filter(i => i.parent_id === null).sort((a, b) => a.sort_order - b.sort_order));

  function childrenOf(parentId: string): MenuItem[] {
    return items.filter(i => i.parent_id === parentId).sort((a, b) => a.sort_order - b.sort_order);
  }

  // Detect external URLs
  function isExternal(item: MenuItem): boolean {
    return item.link_type === 'url' && item.link_target.startsWith('http');
  }
</script>

<nav class="navigation" class:horizontal={orientation === 'horizontal'}>
  <ul class="nav-list">
    {#each rootItems as item (item.id)}
      {@const url = itemUrl(item)}
      {@const children = childrenOf(item.id)}
      <li class="nav-item" class:has-children={children.length > 0}>
        <a
          href={url}
          class="nav-link"
          class:active={isActive(url)}
          target={isExternal(item) ? '_blank' : undefined}
          rel={isExternal(item) ? 'noopener noreferrer' : undefined}
        >
          {item.label}
          {#if isExternal(item)}
            <svg class="external-icon" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
              <polyline points="15 3 21 3 21 9"/>
              <line x1="10" y1="14" x2="21" y2="3"/>
            </svg>
          {/if}
        </a>
        {#if children.length > 0}
          <ul class="nav-submenu">
            {#each children as child (child.id)}
              {@const childUrl = itemUrl(child)}
              <li>
                <a
                  href={childUrl}
                  class="nav-link nav-link--child"
                  class:active={isActive(childUrl)}
                  target={isExternal(child) ? '_blank' : undefined}
                  rel={isExternal(child) ? 'noopener noreferrer' : undefined}
                >
                  {child.label}
                </a>
              </li>
            {/each}
          </ul>
        {/if}
      </li>
    {/each}
  </ul>
</nav>

<style>
  .nav-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .navigation.horizontal .nav-list {
    flex-direction: row;
    gap: var(--space-xs);
    align-items: center;
  }

  .nav-item {
    position: relative;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    text-decoration: none;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .nav-link:hover {
    background: var(--color-bg);
    color: var(--color-text);
  }

  .nav-link.active {
    background: rgba(232, 146, 74, 0.12);
    color: var(--color-primary);
  }

  .nav-link--child {
    font-size: 0.85rem;
    padding-left: calc(var(--space-md) + var(--space-md));
  }

  .nav-submenu {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .external-icon {
    flex-shrink: 0;
    opacity: 0.6;
  }

  /* Horizontal variant: submenu drops down */
  .navigation.horizontal .nav-submenu {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    min-width: 180px;
    padding: var(--space-xs);
    display: none;
    z-index: 50;
  }

  .navigation.horizontal .nav-item.has-children:hover .nav-submenu,
  .navigation.horizontal .nav-item.has-children:focus-within .nav-submenu {
    display: flex;
    flex-direction: column;
  }

  .navigation.horizontal .nav-link--child {
    padding-left: var(--space-md);
  }
</style>
