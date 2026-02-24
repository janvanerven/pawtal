<script lang="ts">
  import type { PageData } from './$types';
  import { formatDate } from '$lib/utils';
  import InView from '$lib/components/InView.svelte';
  import Logo from '$lib/components/Logo.svelte';

  let { data }: { data: PageData } = $props();

  // Pagination helpers for app catalogue mode
  const currentPage = $derived(
    data.type === 'apps' ? (data.apps?.page ?? 1) : 1
  );

  const totalPages = $derived(
    data.type === 'apps' && data.apps
      ? Math.ceil(data.apps.total / data.apps.per_page)
      : 1
  );

  const siteTitle = $derived(data.settings?.site_title || 'Pawtal');
  const siteDescription = $derived(data.settings?.site_description || 'Projects, articles, and more.');

  /** Only allow http/https URLs to prevent javascript: injection. */
  function safeUrl(url: string | null | undefined): string | undefined {
    if (!url) return undefined;
    try {
      const parsed = new URL(url);
      if (parsed.protocol === 'https:' || parsed.protocol === 'http:') return url;
    } catch { /* invalid URL */ }
    return undefined;
  }
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
            {#if safeUrl(app.url)}
              <a href={safeUrl(app.url)} target="_blank" rel="noopener noreferrer" class="app-link btn btn-primary">
                Visit
              </a>
            {:else if app.page_id}
              <a href="/{app.page_id}" class="app-link btn btn-primary">Learn more</a>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#if totalPages > 1}
      <div class="pagination">
        {#if currentPage > 1}
          <a href="?page={currentPage - 1}" class="btn btn-ghost">Previous</a>
        {/if}
        <span class="page-info">Page {currentPage} of {totalPages}</span>
        {#if currentPage < totalPages}
          <a href="?page={currentPage + 1}" class="btn btn-ghost">Next</a>
        {/if}
      </div>
    {/if}
  </section>

{:else}
  <!-- Default: curated homepage -->

  <!-- Hero Section -->
  <section class="hero">
    <div class="hero-pattern"></div>
    <div class="hero-glow hero-glow-1"></div>
    <div class="hero-glow hero-glow-2"></div>
    <div class="hero-content">
      <div class="hero-logo">
        <Logo size={72} />
      </div>
      <h1 class="hero-title">
        <span class="gradient-text">{siteTitle}</span>
      </h1>
      <p class="hero-subtitle">{siteDescription}</p>
      <div class="hero-actions">
        <a href="/articles" class="btn-gradient">Read Articles</a>
        <a href="/apps" class="btn btn-ghost btn-outline">Browse Projects</a>
      </div>
    </div>
    <!-- Floating decorative shapes -->
    <div class="hero-shape hero-shape-1" aria-hidden="true"></div>
    <div class="hero-shape hero-shape-2" aria-hidden="true"></div>
    <div class="hero-shape hero-shape-3" aria-hidden="true"></div>
  </section>

  <!-- Featured Projects -->
  {#if data.featuredApps && data.featuredApps.data.length > 0}
    <InView>
      <section class="section">
        <div class="section-header">
          <div class="section-header-left">
            <div class="section-accent"></div>
            <h2>Featured Projects</h2>
          </div>
          <a href="/apps" class="section-link">View all &rarr;</a>
        </div>
        <div class="projects-grid">
          {#each data.featuredApps.data as app, i (app.id)}
            <InView delay={i * 100}>
              <a
                href={safeUrl(app.url) || (app.page_id ? `/${app.page_id}` : '/apps')}
                class="project-card"
                target={safeUrl(app.url) ? '_blank' : undefined}
                rel={safeUrl(app.url) ? 'noopener noreferrer' : undefined}
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

  <!-- Section divider -->
  <div class="section-divider" aria-hidden="true">
    <span></span><span></span><span></span>
  </div>

  <!-- Latest Articles -->
  {#if data.articles && data.articles.data.length > 0}
    <InView>
      <section class="section">
        <div class="section-header">
          <div class="section-header-left">
            <div class="section-accent"></div>
            <h2>Latest Articles</h2>
          </div>
          <a href="/articles" class="section-link">View all &rarr;</a>
        </div>
        <div class="articles-layout">
          <!-- First article as hero card -->
          {#if data.articles.data.length > 0}
            {@const hero = data.articles.data[0]}
            <InView>
              <a href="/articles/{hero.slug}" class="article-hero-card">
                {#if hero.cover_image_id}
                  <div class="article-hero-cover">
                    <img
                      src="/uploads/{hero.cover_image_id}/medium.webp"
                      alt={hero.title}
                      loading="lazy"
                    />
                  </div>
                {:else}
                  <div class="article-hero-cover article-hero-cover-empty">
                    <Logo size={48} />
                  </div>
                {/if}
                <div class="article-hero-body">
                  <span class="article-badge">Latest</span>
                  <div class="article-meta">
                    <time datetime={hero.publish_at ?? hero.created_at}>
                      {formatDate(hero.publish_at ?? hero.created_at)}
                    </time>
                    {#if hero.reading_time_minutes > 0}
                      <span class="reading-time">{hero.reading_time_minutes} min read</span>
                    {/if}
                  </div>
                  <h3 class="article-hero-title">{hero.title}</h3>
                  {#if hero.short_text}
                    <p class="article-hero-excerpt">{hero.short_text}</p>
                  {/if}
                  <span class="article-read-more">Read article &rarr;</span>
                </div>
              </a>
            </InView>
          {/if}

          <!-- Remaining articles in grid -->
          {#if data.articles.data.length > 1}
            <div class="articles-grid">
              {#each data.articles.data.slice(1) as article, i (article.id)}
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
          {/if}
        </div>
      </section>
    </InView>
  {/if}
{/if}

<style>
  /* ---- Hero ---- */
  .hero {
    text-align: center;
    padding: var(--space-3xl) var(--space-lg) calc(var(--space-3xl) + 2rem);
    position: relative;
    overflow: hidden;
    margin: calc(-1 * var(--space-xl)) calc(-1 * var(--space-lg)) var(--space-2xl);
  }

  /* Dot pattern background */
  .hero-pattern {
    position: absolute;
    inset: 0;
    background-image: radial-gradient(circle, var(--color-primary) 0.75px, transparent 0.75px);
    background-size: 28px 28px;
    opacity: 0.06;
    z-index: 0;
  }

  /* Gradient glow orbs */
  .hero-glow {
    position: absolute;
    border-radius: 50%;
    filter: blur(80px);
    z-index: 0;
  }

  .hero-glow-1 {
    width: 400px;
    height: 400px;
    background: rgba(232, 146, 74, 0.15);
    top: -100px;
    right: -80px;
    animation: float-glow 12s ease-in-out infinite;
  }

  .hero-glow-2 {
    width: 350px;
    height: 350px;
    background: rgba(232, 93, 93, 0.12);
    bottom: -80px;
    left: -60px;
    animation: float-glow 15s ease-in-out infinite reverse;
  }

  @keyframes float-glow {
    0%, 100% { transform: translate(0, 0); }
    50% { transform: translate(30px, -20px); }
  }

  .hero-content {
    position: relative;
    z-index: 1;
  }

  .hero-logo {
    display: inline-flex;
    margin-bottom: var(--space-lg);
    animation: fade-in-up 0.6s ease-out both;
  }

  .hero-title {
    font-size: clamp(2.5rem, 7vw, 5rem);
    font-weight: 700;
    letter-spacing: -0.04em;
    line-height: 1.05;
    margin-bottom: var(--space-lg);
    animation: fade-in-up 0.6s ease-out 0.1s both;
  }

  .gradient-text {
    background: var(--gradient-accent);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .hero-subtitle {
    font-size: clamp(1.05rem, 2vw, 1.3rem);
    color: var(--color-text-muted);
    max-width: 560px;
    margin: 0 auto var(--space-xl);
    line-height: 1.7;
    animation: fade-in-up 0.6s ease-out 0.2s both;
  }

  .hero-actions {
    display: flex;
    gap: var(--space-md);
    justify-content: center;
    flex-wrap: wrap;
    animation: fade-in-up 0.6s ease-out 0.3s both;
  }

  @keyframes fade-in-up {
    from {
      opacity: 0;
      transform: translateY(16px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Floating decorative shapes */
  .hero-shape {
    position: absolute;
    border-radius: var(--radius-md);
    border: 2px solid var(--color-primary);
    opacity: 0.08;
    z-index: 0;
  }

  .hero-shape-1 {
    width: 60px;
    height: 60px;
    top: 20%;
    left: 8%;
    transform: rotate(15deg);
    animation: float-shape 8s ease-in-out infinite;
  }

  .hero-shape-2 {
    width: 40px;
    height: 40px;
    bottom: 25%;
    right: 12%;
    border-radius: 50%;
    border-color: var(--color-accent);
    animation: float-shape 10s ease-in-out infinite reverse;
  }

  .hero-shape-3 {
    width: 28px;
    height: 28px;
    top: 35%;
    right: 20%;
    transform: rotate(45deg);
    animation: float-shape 7s ease-in-out 1s infinite;
  }

  @keyframes float-shape {
    0%, 100% { transform: translateY(0) rotate(var(--r, 15deg)); }
    50% { transform: translateY(-12px) rotate(calc(var(--r, 15deg) + 5deg)); }
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
    box-shadow: 0 8px 25px rgba(232, 146, 74, 0.35);
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

  .section-header-left {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .section-accent {
    width: 48px;
    height: 3px;
    background: var(--gradient-accent);
    border-radius: 2px;
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

  /* Section divider */
  .section-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin-bottom: var(--space-3xl);
  }

  .section-divider span {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--color-primary);
    opacity: 0.2;
  }

  .section-divider span:nth-child(2) {
    opacity: 0.35;
    width: 6px;
    height: 6px;
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
    transition: transform var(--transition-spring), box-shadow var(--transition-normal), border-color var(--transition-fast);
  }

  :global([data-theme="dark"]) .project-card {
    background: rgba(41, 37, 36, 0.6);
    border-color: rgba(61, 50, 41, 0.5);
  }

  .project-card:hover {
    transform: translateY(-4px) scale(1.01);
    box-shadow: var(--shadow-lg);
    border-color: var(--color-primary-light);
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

  /* ---- Articles layout ---- */
  .articles-layout {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  /* Hero article card: horizontal layout */
  .article-hero-card {
    display: grid;
    grid-template-columns: 1fr 1fr;
    border-radius: var(--radius-xl);
    overflow: hidden;
    background: var(--color-surface);
    box-shadow: var(--shadow-md);
    text-decoration: none;
    color: var(--color-text);
    transition: transform var(--transition-spring), box-shadow var(--transition-normal);
    border: 1px solid var(--color-border);
  }

  .article-hero-card:hover {
    transform: translateY(-4px);
    box-shadow: var(--shadow-lg);
    color: var(--color-text);
  }

  .article-hero-cover {
    aspect-ratio: 4/3;
    overflow: hidden;
  }

  .article-hero-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform var(--transition-normal);
  }

  .article-hero-card:hover .article-hero-cover img {
    transform: scale(1.03);
  }

  .article-hero-cover-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, rgba(232,146,74,0.08), rgba(232,93,93,0.08));
  }

  .article-hero-body {
    padding: var(--space-xl) var(--space-xl);
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: var(--space-sm);
  }

  .article-badge {
    display: inline-flex;
    align-self: flex-start;
    padding: 2px 10px;
    border-radius: var(--radius-full);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: var(--gradient-accent);
    color: white;
  }

  .article-hero-title {
    font-size: clamp(1.25rem, 2.5vw, 1.75rem);
    font-weight: 700;
    line-height: 1.25;
  }

  .article-hero-excerpt {
    color: var(--color-text-muted);
    font-size: 0.95rem;
    line-height: 1.65;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .article-read-more {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-primary);
    margin-top: var(--space-sm);
  }

  .article-hero-card:hover .article-read-more {
    color: var(--color-primary-hover);
  }

  /* Regular articles grid */
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
    border: 1px solid transparent;
  }

  .article-card:hover {
    transform: translateY(-4px);
    box-shadow: var(--shadow-lg);
    border-color: var(--color-primary-light);
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
    .article-hero-card {
      grid-template-columns: 1fr;
    }
    .article-hero-cover {
      aspect-ratio: 16/9;
    }
    .hero-shape {
      display: none;
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
