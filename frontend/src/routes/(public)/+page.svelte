<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';
  import InView from '$lib/components/InView.svelte';

  let { data }: { data: PageData } = $props();

  // Pagination helpers for app catalogue mode
  const currentPage = $derived(
    data.type === 'apps' ? (data.apps?.page ?? 1) : 1
  );

  const totalPages = $derived(() => {
    if (data.type === 'apps' && data.apps) {
      return Math.ceil(data.apps.total / data.apps.per_page);
    }
    return 1;
  });

  const siteTitle = $derived(data.settings?.site_title || 'Pawtal');
  const siteDescription = $derived(data.settings?.site_description || 'Projects, articles, and more.');
</script>

<svelte:head>
  <link rel="alternate" type="application/atom+xml" title="{siteTitle} Feed" href="/feed.xml" />
</svelte:head>

{#if data.type === 'page' && data.page}
  <article class="page-content">
    <h1>{data.page.title}</h1>
    <div class="prose">
      {@html data.page.content}
    </div>
  </article>

{:else if data.type === 'apps' && data.apps}
  <section>
    {#if data.settings?.app_catalogue_intro}
      <div class="section-intro prose">
        {@html data.settings.app_catalogue_intro}
      </div>
    {/if}
    <h1 class="section-title">App Catalogue</h1>

    {#if data.apps.data.length === 0}
      <p class="empty-state">No apps available yet.</p>
    {:else}
      <div class="apps-grid">
        {#each data.apps.data as app (app.id)}
          <div class="app-card card">
            {#if app.icon_id}
              <img
                src="/uploads/{app.icon_id}/{app.icon_filename}"
                alt={app.name}
                class="app-icon"
                loading="lazy"
              />
            {:else}
              <div class="app-icon-placeholder"></div>
            {/if}
            <div class="app-info">
              <h3 class="app-name">{app.name}</h3>
              {#if app.description}
                <p class="app-description">{app.description}</p>
              {/if}
            </div>
            {#if app.url}
              <a href={app.url} target="_blank" rel="noopener noreferrer" class="app-link btn btn-primary">
                Visit
              </a>
            {:else if app.page_id}
              <a href="/{app.page_id}" class="app-link btn btn-primary">Learn more</a>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#if totalPages() > 1}
      <div class="pagination">
        {#if currentPage > 1}
          <a href="?page={currentPage - 1}" class="btn btn-ghost">Previous</a>
        {/if}
        <span class="page-info">Page {currentPage} of {totalPages()}</span>
        {#if currentPage < totalPages()}
          <a href="?page={currentPage + 1}" class="btn btn-ghost">Next</a>
        {/if}
      </div>
    {/if}
  </section>

{:else}
  <!-- Default: curated homepage -->

  <!-- Hero Section -->
  <section class="hero">
    <div class="hero-bg"></div>
    <div class="hero-content">
      <h1 class="hero-title">{siteTitle}</h1>
      <p class="hero-subtitle">{siteDescription}</p>
      <div class="hero-actions">
        <a href="/articles" class="btn-gradient">Read Articles</a>
        <a href="/apps" class="btn btn-ghost btn-outline">Browse Projects</a>
      </div>
    </div>
  </section>

  <!-- Featured Projects -->
  {#if data.featuredApps && data.featuredApps.data.length > 0}
    <InView>
      <section class="section">
        <div class="section-header">
          <h2>Featured Projects</h2>
          <a href="/apps" class="section-link">View all &rarr;</a>
        </div>
        <div class="projects-grid">
          {#each data.featuredApps.data as app, i (app.id)}
            <InView delay={i * 100}>
              <a
                href={app.url || (app.page_id ? `/${app.page_id}` : '/apps')}
                class="project-card"
                target={app.url ? '_blank' : undefined}
                rel={app.url ? 'noopener noreferrer' : undefined}
              >
                {#if app.icon_id}
                  <img
                    src="/uploads/{app.icon_id}/{app.icon_filename}"
                    alt={app.name}
                    class="project-icon"
                    loading="lazy"
                  />
                {:else}
                  <div class="project-icon-placeholder"></div>
                {/if}
                <h3 class="project-name">{app.name}</h3>
                {#if app.description}
                  <p class="project-description">{app.description}</p>
                {/if}
              </a>
            </InView>
          {/each}
        </div>
      </section>
    </InView>
  {/if}

  <!-- Latest Articles -->
  {#if data.articles && data.articles.data.length > 0}
    <InView>
      <section class="section">
        <div class="section-header">
          <h2>Latest Articles</h2>
          <a href="/articles" class="section-link">View all &rarr;</a>
        </div>
        <div class="articles-grid">
          {#each data.articles.data as article, i (article.id)}
            <InView delay={i * 80}>
              <a href="/articles/{article.slug}" class="article-card">
                {#if article.cover_image_id}
                  <div class="article-cover">
                    <img
                      src="/uploads/{article.cover_image_id}/medium.webp"
                      alt={article.title}
                      loading="lazy"
                    />
                  </div>
                {/if}
                <div class="article-card-body">
                  <div class="article-meta">
                    <time datetime={article.publish_at ?? article.created_at}>
                      {formatDate(article.publish_at ?? article.created_at)}
                    </time>
                    {#if article.reading_time_minutes > 0}
                      <span class="reading-time">{article.reading_time_minutes} min read</span>
                    {/if}
                  </div>
                  <h3 class="article-title">{article.title}</h3>
                  {#if article.short_text}
                    <p class="article-excerpt">{article.short_text}</p>
                  {/if}
                </div>
              </a>
            </InView>
          {/each}
        </div>
      </section>
    </InView>
  {/if}
{/if}

<style>
  /* ---- Hero ---- */
  .hero {
    text-align: center;
    padding: var(--space-3xl) var(--space-lg);
    position: relative;
    overflow: hidden;
    margin: calc(-1 * var(--space-xl)) calc(-1 * var(--space-lg)) var(--space-2xl);
  }

  .hero-bg {
    position: absolute;
    inset: -50%;
    background: conic-gradient(
      from 0deg at 50% 50%,
      rgba(232, 146, 74, 0.08) 0deg,
      rgba(232, 93, 93, 0.06) 120deg,
      rgba(123, 166, 140, 0.05) 240deg,
      rgba(232, 146, 74, 0.08) 360deg
    );
    animation: rotate-gradient 25s linear infinite;
    z-index: 0;
  }

  @keyframes rotate-gradient {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .hero-content {
    position: relative;
    z-index: 1;
  }

  .hero-title {
    font-size: clamp(2.5rem, 6vw, 4.5rem);
    font-weight: 300;
    letter-spacing: -0.03em;
    line-height: 1.1;
    margin-bottom: var(--space-lg);
  }

  .hero-subtitle {
    font-size: clamp(1rem, 2vw, 1.25rem);
    color: var(--color-text-muted);
    max-width: 600px;
    margin: 0 auto var(--space-xl);
    line-height: 1.6;
  }

  .hero-actions {
    display: flex;
    gap: var(--space-md);
    justify-content: center;
    flex-wrap: wrap;
  }

  .btn-gradient {
    display: inline-flex;
    align-items: center;
    background: var(--gradient-accent);
    color: white;
    padding: var(--space-md) var(--space-xl);
    border-radius: var(--radius-full);
    font-weight: 600;
    font-size: 1rem;
    text-decoration: none;
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  }

  .btn-gradient:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(232, 146, 74, 0.3);
    color: white;
  }

  .btn-outline {
    padding: var(--space-md) var(--space-xl);
    border-radius: var(--radius-full);
    font-size: 1rem;
    text-decoration: none;
  }

  /* ---- Sections ---- */
  .section {
    margin-bottom: var(--space-3xl);
  }

  .section-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-xl);
  }

  .section-header h2 {
    font-size: clamp(1.5rem, 3vw, 2rem);
  }

  .section-link {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-primary);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  .section-link:hover {
    color: var(--color-primary-hover);
  }

  /* ---- Featured Projects ---- */
  .projects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--space-lg);
  }

  .project-card {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(232, 222, 212, 0.5);
    border-radius: var(--radius-xl);
    padding: var(--space-xl);
    text-decoration: none;
    color: var(--color-text);
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-md);
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  }

  :global([data-theme="dark"]) .project-card {
    background: rgba(41, 37, 36, 0.6);
    border-color: rgba(61, 50, 41, 0.5);
  }

  .project-card:hover {
    transform: translateY(-4px) scale(1.01);
    box-shadow: var(--shadow-lg);
    color: var(--color-text);
  }

  .project-icon {
    width: 80px;
    height: 80px;
    border-radius: var(--radius-lg);
    object-fit: contain;
  }

  .project-icon-placeholder {
    width: 80px;
    height: 80px;
    border-radius: var(--radius-lg);
    background: var(--gradient-accent);
    opacity: 0.15;
  }

  .project-name {
    font-size: 1.1rem;
    font-weight: 600;
  }

  .project-description {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* ---- Latest Articles ---- */
  .articles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: var(--space-lg);
  }

  .article-card {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--color-surface);
    box-shadow: var(--shadow-sm);
    text-decoration: none;
    color: var(--color-text);
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  }

  .article-card:hover {
    transform: translateY(-4px);
    box-shadow: var(--shadow-lg);
    color: var(--color-text);
  }

  .article-cover {
    width: 100%;
    aspect-ratio: 16/9;
    overflow: hidden;
  }

  .article-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform var(--transition-normal);
  }

  .article-card:hover .article-cover img {
    transform: scale(1.03);
  }

  .article-card-body {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    flex: 1;
  }

  .article-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .reading-time::before {
    content: '\00B7';
    margin-right: var(--space-sm);
  }

  .article-title {
    font-size: 1.15rem;
    font-weight: 600;
    line-height: 1.3;
  }

  .article-excerpt {
    color: var(--color-text-muted);
    font-size: 0.875rem;
    line-height: 1.6;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* ---- App Catalogue (legacy mode) ---- */
  .section-title {
    margin-bottom: var(--space-xl);
    color: var(--color-text);
  }

  .section-intro {
    margin-bottom: var(--space-xl);
    padding: var(--space-lg);
    background: var(--color-surface);
    border-radius: var(--radius-md);
    border-left: 4px solid var(--color-primary);
  }

  .page-content h1 {
    margin-bottom: var(--space-xl);
  }

  .apps-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .app-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-md);
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  }

  .app-card:hover {
    box-shadow: var(--shadow-md);
    transform: translateY(-2px);
  }

  .app-icon {
    width: 72px;
    height: 72px;
    border-radius: var(--radius-md);
    object-fit: contain;
  }

  .app-icon-placeholder {
    width: 72px;
    height: 72px;
    border-radius: var(--radius-md);
    background: var(--gradient-accent);
    opacity: 0.15;
  }

  .app-info { flex: 1; }
  .app-name { font-size: 1rem; margin-bottom: var(--space-xs); }
  .app-description { font-size: 0.85rem; color: var(--color-text-muted); line-height: 1.5; }
  .app-link { width: 100%; justify-content: center; }

  /* Pagination */
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    margin-top: var(--space-xl);
  }

  .page-info {
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  /* Empty state */
  .empty-state {
    text-align: center;
    color: var(--color-text-muted);
    padding: var(--space-2xl) 0;
    font-size: 1rem;
  }

  @media (max-width: 768px) {
    .apps-grid {
      grid-template-columns: repeat(2, 1fr);
    }
    .hero {
      padding: var(--space-2xl) var(--space-md);
    }
  }

  @media (max-width: 480px) {
    .apps-grid {
      grid-template-columns: 1fr;
    }
    .articles-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
