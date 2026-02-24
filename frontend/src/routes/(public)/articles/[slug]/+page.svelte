<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';
  import { page } from '$app/stores';
  import { toasts } from '$lib/stores/toasts';
  import InView from '$lib/components/InView.svelte';

  let { data }: { data: PageData } = $props();

  let progress = $state(0);

  function handleScroll() {
    const scrollable = document.documentElement.scrollHeight - window.innerHeight;
    progress = scrollable > 0 ? (window.scrollY / scrollable) * 100 : 0;
  }

  const coverImageUrl = $derived(
    data.article.cover_image_id
      ? `/uploads/${data.article.cover_image_id}/large.webp`
      : null
  );

  async function copyLink() {
    try {
      await navigator.clipboard.writeText(window.location.href);
      toasts.success('Link copied to clipboard');
    } catch {
      toasts.error('Failed to copy link');
    }
  }

  function shareTwitter() {
    const url = encodeURIComponent(window.location.href);
    const text = encodeURIComponent(data.article.title);
    window.open(`https://twitter.com/intent/tweet?url=${url}&text=${text}`, '_blank');
  }

  function shareLinkedIn() {
    const url = encodeURIComponent(window.location.href);
    window.open(`https://www.linkedin.com/sharing/share-offsite/?url=${url}`, '_blank');
  }
</script>

<svelte:window onscroll={handleScroll} />

<svelte:head>
  <title>{data.article.title}</title>
  <meta name="description" content={data.article.short_text || ''} />
  <meta property="og:title" content={data.article.title} />
  <meta property="og:description" content={data.article.short_text || ''} />
  <meta property="og:type" content="article" />
  <meta property="og:url" content={$page.url.href} />
  {#if coverImageUrl}
    <meta property="og:image" content={coverImageUrl} />
  {/if}
</svelte:head>

<!-- Reading progress bar -->
<div class="progress-bar" style="width: {progress}%"></div>

<article class="article-view">
  <!-- Cover image -->
  {#if coverImageUrl}
    <div class="article-cover">
      <img src={coverImageUrl} alt={data.article.title} />
    </div>
  {/if}

  <div class="article-layout">
    <!-- Table of contents (desktop sidebar) -->
    {#if data.toc && data.toc.length > 0}
      <aside class="toc-sidebar">
        <nav class="toc">
          <h4 class="toc-title">Contents</h4>
          <ul class="toc-list">
            {#each data.toc as item}
              <li class="toc-item" class:toc-h3={item.level === 3}>
                <a href="#{item.id}" class="toc-link">{item.text}</a>
              </li>
            {/each}
          </ul>
        </nav>
      </aside>
    {/if}

    <div class="article-content">
      <header class="article-header">
        <div class="article-meta">
          <time datetime={data.article.publish_at ?? data.article.created_at}>
            {formatDate(data.article.publish_at ?? data.article.created_at)}
          </time>
          {#if data.article.reading_time_minutes > 0}
            <span class="reading-time">{data.article.reading_time_minutes} min read</span>
          {/if}
        </div>
        <h1>{data.article.title}</h1>
        {#if data.article.short_text}
          <p class="article-intro">{data.article.short_text}</p>
        {/if}
      </header>

      {#if data.article.content}
        <div class="prose article-body">
          {@html data.article.content}
        </div>
      {/if}

      <!-- Share buttons -->
      <div class="share-section">
        <span class="share-label">Share</span>
        <div class="share-buttons">
          <button class="share-btn" onclick={copyLink} title="Copy link">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
            Copy link
          </button>
          <button class="share-btn" onclick={shareTwitter} title="Share on X">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>
            X
          </button>
          <button class="share-btn" onclick={shareLinkedIn} title="Share on LinkedIn">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/></svg>
            LinkedIn
          </button>
        </div>
      </div>

      <footer class="article-footer">
        <a href="/articles" class="back-link">&larr; Back to articles</a>
      </footer>
    </div>
  </div>

  <!-- Related articles -->
  {#if data.related && data.related.length > 0}
    <InView>
      <section class="related-section">
        <h2 class="related-title">Related Articles</h2>
        <div class="related-grid">
          {#each data.related as article, i (article.id)}
            <InView delay={i * 100}>
              <a href="/articles/{article.slug}" class="related-card">
                {#if article.cover_image_id}
                  <div class="related-cover">
                    <img
                      src="/uploads/{article.cover_image_id}/medium.webp"
                      alt={article.title}
                      loading="lazy"
                    />
                  </div>
                {/if}
                <div class="related-body">
                  <time datetime={article.publish_at ?? article.created_at}>
                    {formatDate(article.publish_at ?? article.created_at)}
                  </time>
                  <h3>{article.title}</h3>
                </div>
              </a>
            </InView>
          {/each}
        </div>
      </section>
    </InView>
  {/if}
</article>

<style>
  /* Progress bar */
  .progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    height: 3px;
    background: var(--gradient-accent);
    z-index: 100;
    transition: width 50ms linear;
    pointer-events: none;
  }

  .article-view {
    max-width: 100%;
  }

  /* Cover image */
  .article-cover {
    margin: calc(-1 * var(--space-xl)) calc(-1 * var(--space-lg)) var(--space-xl);
    max-height: 400px;
    overflow: hidden;
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
  }

  .article-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* Layout with TOC sidebar */
  .article-layout {
    display: flex;
    gap: var(--space-2xl);
    max-width: var(--width-wide);
  }

  .article-content {
    flex: 1;
    min-width: 0;
    max-width: var(--width-prose);
  }

  /* TOC Sidebar */
  .toc-sidebar {
    width: 220px;
    flex-shrink: 0;
    order: 1;
  }

  .toc {
    position: sticky;
    top: calc(64px + var(--space-xl));
  }

  .toc-title {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin-bottom: var(--space-md);
  }

  .toc-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .toc-item.toc-h3 {
    padding-left: var(--space-md);
  }

  .toc-link {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    text-decoration: none;
    line-height: 1.4;
    display: block;
    padding: 2px 0;
    transition: color var(--transition-fast);
  }

  .toc-link:hover {
    color: var(--color-primary);
  }

  /* Article header */
  .article-header {
    margin-bottom: var(--space-xl);
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--color-border);
  }

  .article-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 0.85rem;
    color: var(--color-text-muted);
    margin-bottom: var(--space-md);
  }

  .reading-time::before {
    content: '\00B7';
    margin-right: var(--space-sm);
  }

  .article-header h1 {
    margin-bottom: var(--space-md);
  }

  .article-intro {
    font-size: 1.1rem;
    color: var(--color-text-muted);
    line-height: 1.7;
  }

  .article-body {
    margin-bottom: var(--space-2xl);
  }

  /* Share section */
  .share-section {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-lg) 0;
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
    margin-bottom: var(--space-lg);
  }

  .share-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .share-buttons {
    display: flex;
    gap: var(--space-sm);
  }

  .share-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 6px 12px;
    border-radius: var(--radius-full);
    font-size: 0.8rem;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    background: transparent;
    transition: all var(--transition-fast);
    cursor: pointer;
  }

  .share-btn:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  /* Article footer */
  .article-footer {
    padding-top: var(--space-lg);
  }

  .back-link {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  .back-link:hover {
    color: var(--color-primary);
  }

  /* Related articles */
  .related-section {
    margin-top: var(--space-3xl);
    padding-top: var(--space-2xl);
    border-top: 1px solid var(--color-border);
  }

  .related-title {
    margin-bottom: var(--space-xl);
    font-size: 1.5rem;
  }

  .related-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-lg);
  }

  .related-card {
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--color-surface);
    box-shadow: var(--shadow-sm);
    text-decoration: none;
    color: var(--color-text);
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  }

  .related-card:hover {
    transform: translateY(-3px);
    box-shadow: var(--shadow-md);
    color: var(--color-text);
  }

  .related-cover {
    aspect-ratio: 16/9;
    overflow: hidden;
  }

  .related-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .related-body {
    padding: var(--space-md);
  }

  .related-body time {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .related-body h3 {
    font-size: 1rem;
    margin-top: var(--space-xs);
    line-height: 1.3;
  }

  @media (max-width: 1024px) {
    .toc-sidebar {
      display: none;
    }
  }

  @media (max-width: 768px) {
    .article-cover {
      margin: calc(-1 * var(--space-md)) calc(-1 * var(--space-md)) var(--space-lg);
      border-radius: 0;
    }
  }
</style>
