# Pawtal Brand Reinvention — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform Pawtal from a generic CMS into a bold, animated product/project hub with a premium design system, rich project showcases, enhanced reading experience, and a modern editor.

**Architecture:** The existing Rust/Axum + SvelteKit + SQLite stack is retained. Changes are additive: new DB columns via migrations, new API endpoints, new frontend components, and a CSS design system overhaul. The work is organized into 8 phases that can be executed sequentially, each producing a working state.

**Tech Stack:** Rust/Axum (backend), SvelteKit 5 (frontend), SQLite (DB), TipTap 3 (editor), Space Grotesk + DM Sans (fonts), Shiki (syntax highlighting)

---

## Phase 1: Design System Foundation

Swap fonts, update CSS custom properties, add animation tokens, and update the dark mode palette. Every subsequent phase builds on this foundation.

### Task 1.1: Install new font packages

**Files:**
- Modify: `frontend/package.json`

**Step 1: Install new fonts and remove old ones**

Run:
```bash
cd frontend && npm install @fontsource/space-grotesk @fontsource/dm-sans && npm uninstall @fontsource/inter @fontsource/nunito
```

**Step 2: Verify packages installed**

Run: `cd frontend && ls node_modules/@fontsource/space-grotesk node_modules/@fontsource/dm-sans`
Expected: directories exist

**Step 3: Commit**

```bash
git add frontend/package.json frontend/package-lock.json
git commit -m "chore: swap fonts to Space Grotesk + DM Sans"
```

---

### Task 1.2: Update global CSS design tokens

**Files:**
- Modify: `frontend/src/app.css`

**Step 1: Replace font imports and CSS custom properties**

Replace the font imports at the top of `app.css`:

```css
@import '@fontsource/space-grotesk/300.css';
@import '@fontsource/space-grotesk/500.css';
@import '@fontsource/space-grotesk/700.css';
@import '@fontsource/dm-sans/400.css';
@import '@fontsource/dm-sans/500.css';
@import '@fontsource/dm-sans/700.css';
```

Update the `:root` block with new tokens:

```css
:root {
  /* Colors */
  --color-primary: #E8924A;
  --color-primary-hover: #D47E3A;
  --color-primary-light: #F5C89A;
  --color-secondary: #7BA68C;
  --color-secondary-hover: #6B967C;
  --color-accent: #E85D5D;
  --color-accent-hover: #D54D4D;
  --color-bg: #FFF8F0;
  --color-surface: #FFFFFF;
  --color-surface-elevated: #FFFFFF;
  --color-text: #3D3229;
  --color-text-muted: #8A7D72;
  --color-text-light: #B5A99E;
  --color-border: #E8DED4;
  --color-success: #4CAF50;
  --color-warning: #FF9800;
  --color-error: #E85D5D;

  /* Gradient */
  --gradient-accent: linear-gradient(135deg, #E8924A, #E85D5D);

  /* Radii */
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 20px;
  --radius-xl: 28px;
  --radius-full: 9999px;

  /* Shadows */
  --shadow-sm: 0 1px 3px rgba(61,50,41,0.08);
  --shadow-md: 0 4px 12px rgba(61,50,41,0.12);
  --shadow-lg: 0 8px 24px rgba(61,50,41,0.16);

  /* Fonts */
  --font-heading: 'Space Grotesk', sans-serif;
  --font-body: 'DM Sans', sans-serif;

  /* Spacing */
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
  --space-3xl: 64px;

  /* Transitions */
  --transition-fast: 150ms ease;
  --transition-normal: 250ms ease;
  --transition-spring: 300ms cubic-bezier(0.34, 1.56, 0.64, 1);

  /* Content widths */
  --width-wide: 1280px;
  --width-content: 900px;
  --width-prose: 720px;
}
```

Update dark mode to premium palette:

```css
[data-theme="dark"] {
  --color-bg: #0C0A09;
  --color-surface: #1C1917;
  --color-surface-elevated: #292524;
  --color-text: #F5F0EB;
  --color-text-muted: #A89888;
  --color-text-light: #6A5D52;
  --color-border: #3D3229;
  --shadow-sm: 0 1px 3px rgba(0,0,0,0.3);
  --shadow-md: 0 4px 12px rgba(0,0,0,0.4);
  --shadow-lg: 0 8px 24px rgba(0,0,0,0.5);
}
```

**Step 2: Update heading styles for new typography**

Replace the heading rules:

```css
h1, h2, h3, h4, h5, h6 {
  font-family: var(--font-heading);
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: -0.02em;
}

h1 { font-size: clamp(2rem, 4vw, 3rem); }
h2 { font-size: clamp(1.5rem, 3vw, 2rem); }
h3 { font-size: 1.25rem; }
```

**Step 3: Add smooth color transitions to all themed properties**

Add to the `html` rule:

```css
html {
  font-family: var(--font-body);
  font-size: 16px;
  color: var(--color-text);
  background-color: var(--color-bg);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  transition: background-color var(--transition-normal), color var(--transition-normal);
}
```

**Step 4: Verify the frontend builds without errors**

Run: `cd frontend && npm run check`
Expected: no errors

**Step 5: Commit**

```bash
git add frontend/src/app.css
git commit -m "feat: overhaul design tokens — new fonts, premium dark mode, animation tokens"
```

---

### Task 1.3: Update .prose styles for new typography

**Files:**
- Modify: `frontend/src/app.css` (the `.prose` section)

**Step 1: Update prose heading sizes and spacing**

Update the `.prose h1` through `.prose h4` rules to use the new tighter line-height and letter-spacing from the design system:

```css
.prose h1 { font-size: clamp(1.75rem, 3vw, 2.25rem); letter-spacing: -0.02em; }
.prose h2 { font-size: clamp(1.375rem, 2.5vw, 1.75rem); letter-spacing: -0.01em; }
.prose h3 { font-size: 1.25rem; }
.prose h4 { font-size: 1.1rem; }
```

**Step 2: Update blockquote to use accent gradient border**

```css
.prose blockquote {
  border-left: 4px solid var(--color-primary);
  padding: var(--space-md) var(--space-lg);
  margin: var(--space-lg) 0;
  background: var(--color-surface-elevated);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  color: var(--color-text-muted);
  font-style: italic;
}
```

**Step 3: Commit**

```bash
git add frontend/src/app.css
git commit -m "feat: update prose styles for new design system"
```

---

## Phase 2: Reusable UI Components

Build the shared component library needed by all subsequent phases.

### Task 2.1: Create InView scroll-reveal component

**Files:**
- Create: `frontend/src/lib/components/InView.svelte`

**Step 1: Create the InView component**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    threshold?: number;
    rootMargin?: string;
    once?: boolean;
    class?: string;
    delay?: number;
  }

  let { threshold = 0.1, rootMargin = '0px', once = true, class: className = '', delay = 0 }: Props = $props();

  let element: HTMLElement;
  let visible = $state(false);

  onMount(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          if (delay > 0) {
            setTimeout(() => { visible = true; }, delay);
          } else {
            visible = true;
          }
          if (once) observer.unobserve(element);
        } else if (!once) {
          visible = false;
        }
      },
      { threshold, rootMargin }
    );
    observer.observe(element);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={element}
  class="in-view {className}"
  class:visible
>
  {@render children()}
</div>

{#snippet children()}{/snippet}

<style>
  .in-view {
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.6s ease-out, transform 0.6s ease-out;
  }
  .in-view.visible {
    opacity: 1;
    transform: translateY(0);
  }
</style>
```

Wait — Svelte 5 snippets work differently. Let me use the correct children pattern:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    threshold?: number;
    rootMargin?: string;
    once?: boolean;
    class?: string;
    delay?: number;
    children: Snippet;
  }

  let { threshold = 0.1, rootMargin = '0px', once = true, class: className = '', delay = 0, children }: Props = $props();

  let element: HTMLElement;
  let visible = $state(false);

  onMount(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          if (delay > 0) {
            setTimeout(() => { visible = true; }, delay);
          } else {
            visible = true;
          }
          if (once) observer.unobserve(element);
        } else if (!once) {
          visible = false;
        }
      },
      { threshold, rootMargin }
    );
    observer.observe(element);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={element}
  class="in-view {className}"
  class:visible
>
  {@render children()}
</div>

<style>
  .in-view {
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.6s ease-out, transform 0.6s ease-out;
  }
  .in-view.visible {
    opacity: 1;
    transform: translateY(0);
  }
</style>
```

**Step 2: Commit**

```bash
git add frontend/src/lib/components/InView.svelte
git commit -m "feat: add InView scroll-reveal component"
```

---

### Task 2.2: Create Toast notification system

**Files:**
- Create: `frontend/src/lib/components/Toast.svelte`
- Create: `frontend/src/lib/stores/toasts.ts`

**Step 1: Create the toast store**

```typescript
// frontend/src/lib/stores/toasts.ts
import { writable } from 'svelte/store';

export interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info';
  duration?: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  function add(message: string, type: Toast['type'] = 'info', duration = 4000) {
    const id = crypto.randomUUID();
    update(toasts => [...toasts, { id, message, type, duration }]);
    if (duration > 0) {
      setTimeout(() => remove(id), duration);
    }
    return id;
  }

  function remove(id: string) {
    update(toasts => toasts.filter(t => t.id !== id));
  }

  return {
    subscribe,
    success: (msg: string) => add(msg, 'success'),
    error: (msg: string) => add(msg, 'error'),
    info: (msg: string) => add(msg, 'info'),
    remove,
  };
}

export const toasts = createToastStore();
```

**Step 2: Create the Toast component**

```svelte
<!-- frontend/src/lib/components/Toast.svelte -->
<script lang="ts">
  import { toasts } from '$lib/stores/toasts';
  import { fly } from 'svelte/transition';
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div
      class="toast toast-{toast.type}"
      transition:fly={{ x: 300, duration: 300 }}
    >
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" onclick={() => toasts.remove(toast.id)}>&times;</button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: var(--space-lg);
    right: var(--space-lg);
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    font-size: 0.875rem;
    font-weight: 500;
    backdrop-filter: blur(12px);
  }

  .toast-success {
    background: rgba(76, 175, 80, 0.95);
    color: white;
  }
  .toast-error {
    background: rgba(232, 93, 93, 0.95);
    color: white;
  }
  .toast-info {
    background: var(--color-surface-elevated);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }

  .toast-message { flex: 1; }

  .toast-close {
    font-size: 1.25rem;
    line-height: 1;
    opacity: 0.7;
    color: inherit;
    padding: 0;
    background: none;
  }
  .toast-close:hover { opacity: 1; }
</style>
```

**Step 3: Commit**

```bash
git add frontend/src/lib/stores/toasts.ts frontend/src/lib/components/Toast.svelte
git commit -m "feat: add toast notification system"
```

---

### Task 2.3: Create ConfirmDialog modal component

**Files:**
- Create: `frontend/src/lib/components/ConfirmDialog.svelte`

**Step 1: Create the component**

```svelte
<!-- frontend/src/lib/components/ConfirmDialog.svelte -->
<script lang="ts">
  import { fly, fade } from 'svelte/transition';

  interface Props {
    open: boolean;
    title?: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: 'danger' | 'default';
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    open = $bindable(false),
    title = 'Confirm',
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    variant = 'default',
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleConfirm() {
    onConfirm();
    open = false;
  }

  function handleCancel() {
    onCancel();
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleCancel();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog-overlay" transition:fade={{ duration: 200 }} onclick={handleCancel} onkeydown={handleKeydown}>
    <div class="dialog-content" transition:fly={{ y: 20, duration: 250 }} onclick|stopPropagation>
      <h3 class="dialog-title">{title}</h3>
      <p class="dialog-message">{message}</p>
      <div class="dialog-actions">
        <button class="btn btn-ghost" onclick={handleCancel}>{cancelLabel}</button>
        <button class="btn {variant === 'danger' ? 'btn-danger' : 'btn-primary'}" onclick={handleConfirm}>
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .dialog-content {
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    padding: var(--space-xl);
    max-width: 440px;
    width: 90%;
    box-shadow: var(--shadow-lg);
  }

  .dialog-title {
    font-size: 1.125rem;
    margin-bottom: var(--space-sm);
  }

  .dialog-message {
    color: var(--color-text-muted);
    font-size: 0.9rem;
    line-height: 1.6;
    margin-bottom: var(--space-lg);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
</style>
```

**Step 2: Commit**

```bash
git add frontend/src/lib/components/ConfirmDialog.svelte
git commit -m "feat: add ConfirmDialog modal component"
```

---

## Phase 3: Backend — New Fields & Endpoints

Database migrations and new API endpoints needed for the redesign.

### Task 3.1: Add cover_image_id, reading_time_minutes to articles

**Files:**
- Create: `backend/migrations/002_article_cover_and_reading_time.sql`
- Modify: `backend/src/db/models.rs` — add fields to `Article`, `CreateArticle`, `UpdateArticle`
- Modify: `backend/src/services/articles.rs` — compute reading time, include cover image
- Modify: `backend/src/api/articles.rs` — update SQL queries

**Step 1: Create the migration**

```sql
-- backend/migrations/002_article_cover_and_reading_time.sql
-- Adds cover image and reading time support to articles.
ALTER TABLE articles ADD COLUMN cover_image_id TEXT REFERENCES media(id) ON DELETE SET NULL;
ALTER TABLE articles ADD COLUMN reading_time_minutes INTEGER NOT NULL DEFAULT 0;
```

**Step 2: Update the Article model in models.rs**

Add after `trashed_at` in the `Article` struct:
```rust
pub cover_image_id: Option<String>,
pub reading_time_minutes: i32,
```

Add `cover_image_id: Option<String>` to both `CreateArticle` and `UpdateArticle` structs.

**Step 3: Update article service — reading time calculation**

Add a helper function in `services/articles.rs`:
```rust
/// Estimates reading time from HTML content at ~200 words per minute.
fn estimate_reading_time(html: &str) -> i32 {
    // Strip HTML tags for a rough word count
    let text: String = html.chars().fold((String::new(), false), |(mut acc, in_tag), c| {
        match c {
            '<' => (acc, true),
            '>' => { acc.push(' '); (acc, false) },
            _ if !in_tag => { acc.push(c); (acc, false) },
            _ => (acc, in_tag),
        }
    }).0;
    let word_count = text.split_whitespace().count();
    ((word_count as f64) / 200.0).ceil().max(1.0) as i32
}
```

**Step 4: Update create_article to include new fields**

In the `create_article` function, compute reading time from content and include `cover_image_id` in the INSERT:
```rust
let reading_time = estimate_reading_time(&content);

sqlx::query(
    "INSERT INTO articles \
         (id, title, slug, short_text, content, status, publish_at, author_id, cover_image_id, reading_time_minutes) \
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
)
.bind(&id)
.bind(&input.title)
.bind(&slug)
.bind(&short_text)
.bind(&content)
.bind(&status)
.bind(&input.publish_at)
.bind(author_id)
.bind(&input.cover_image_id)
.bind(reading_time)
.execute(pool)
.await?;
```

**Step 5: Update update_article similarly**

Merge `cover_image_id`:
```rust
let cover_image_id = if input.cover_image_id.is_some() {
    input.cover_image_id
} else {
    existing.cover_image_id.clone()
};
let reading_time = estimate_reading_time(&content);
```

Update the UPDATE SQL to include `cover_image_id = ?, reading_time_minutes = ?`.

**Step 6: Update ALL SELECT queries to include new columns**

Every `SELECT ... FROM articles` in `services/articles.rs` must now include `cover_image_id, reading_time_minutes`. There are queries in:
- `list_articles` (2 queries — with/without status filter)
- `list_published_articles`
- `get_article`
- `get_article_by_slug`

**Step 7: Build and verify**

Run: `cd backend && cargo build`
Expected: compiles without errors

**Step 8: Commit**

```bash
git add backend/migrations/002_article_cover_and_reading_time.sql backend/src/db/models.rs backend/src/services/articles.rs
git commit -m "feat: add cover image and reading time to articles"
```

---

### Task 3.2: Add template field to pages

**Files:**
- Create: `backend/migrations/003_page_template.sql`
- Modify: `backend/src/db/models.rs` — add `template` to `Page`, `CreatePage`, `UpdatePage`
- Modify: `backend/src/services/pages.rs` — include template in queries

**Step 1: Create migration**

```sql
-- backend/migrations/003_page_template.sql
ALTER TABLE pages ADD COLUMN template TEXT NOT NULL DEFAULT 'default' CHECK (template IN ('default', 'project'));
```

**Step 2: Add `template` field to Page model**

```rust
pub template: String,
```

Add `template: Option<String>` to `CreatePage` and `UpdatePage`.

**Step 3: Update all page queries to include template column**

Every `SELECT ... FROM pages` in `services/pages.rs` must include `template`.

**Step 4: Update create/update logic**

In `create_page`: add `template` to INSERT with default `"default"`.
In `update_page`: merge `template` like other optional fields.

**Step 5: Build and verify**

Run: `cd backend && cargo build`

**Step 6: Commit**

```bash
git add backend/migrations/003_page_template.sql backend/src/db/models.rs backend/src/services/pages.rs
git commit -m "feat: add template field to pages for project showcase support"
```

---

### Task 3.3: Add related articles endpoint

**Files:**
- Modify: `backend/src/services/articles.rs` — add `get_related_articles`
- Modify: `backend/src/api/articles.rs` — add `public_related` handler
- Modify: `backend/src/main.rs` — register route

**Step 1: Add service function**

```rust
/// Returns up to `limit` published articles that share at least one category
/// with the given article, excluding the article itself. Ordered by recency.
pub async fn get_related_articles(
    pool: &SqlitePool,
    article_id: &str,
    limit: i64,
) -> AppResult<Vec<Article>> {
    let articles = sqlx::query_as::<_, Article>(
        "SELECT DISTINCT a.id, a.title, a.slug, a.short_text, a.content, a.status, \
                a.publish_at, a.author_id, a.created_at, a.updated_at, a.trashed_at, \
                a.cover_image_id, a.reading_time_minutes \
         FROM articles a \
         INNER JOIN article_categories ac ON ac.article_id = a.id \
         WHERE ac.category_id IN (SELECT category_id FROM article_categories WHERE article_id = ?) \
           AND a.id != ? \
           AND a.status = 'published' \
         ORDER BY a.created_at DESC \
         LIMIT ?",
    )
    .bind(article_id)
    .bind(article_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(articles)
}
```

**Step 2: Add API handler**

```rust
/// `GET /api/articles/:slug/related`
pub async fn public_related(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Vec<Article>>> {
    let article = svc::get_article_by_slug(&state.db, &slug).await?;
    let related = svc::get_related_articles(&state.db, &article.id, 4).await?;
    Ok(Json(related))
}
```

**Step 3: Register route in main.rs**

Add to `public_routes`:
```rust
.route("/api/articles/{slug}/related", get(api::articles::public_related))
```

**Step 4: Build and verify**

Run: `cd backend && cargo build`

**Step 5: Commit**

```bash
git add backend/src/services/articles.rs backend/src/api/articles.rs backend/src/main.rs
git commit -m "feat: add related articles endpoint"
```

---

### Task 3.4: Add RSS feed endpoint

**Files:**
- Modify: `backend/src/main.rs` — register route
- Create: `backend/src/api/feed.rs` — RSS handler
- Modify: `backend/src/api/mod.rs` — declare module

**Step 1: Create feed.rs**

```rust
//! RSS/Atom feed endpoint.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
};
use crate::db::models::PaginationParams;
use crate::services::articles as svc;
use crate::services::settings;
use crate::AppState;

/// `GET /feed.xml` — Returns an Atom feed of published articles.
pub async fn atom_feed(State(state): State<AppState>) -> impl IntoResponse {
    let params = PaginationParams { page: Some(1), per_page: Some(20) };

    let articles = match svc::list_published_articles(&state.db, &params).await {
        Ok(r) => r.data,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "feed generation failed").into_response(),
    };

    let site_settings = settings::get_public_settings(&state.db).await.unwrap_or_default();
    let site_title = site_settings.get("site_title").cloned().unwrap_or_else(|| "Pawtal".to_string());
    let base_url = state.config.base_url.trim_end_matches('/');

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", escape_xml(&site_title)));
    xml.push_str(&format!("  <link href=\"{}/feed.xml\" rel=\"self\"/>\n", base_url));
    xml.push_str(&format!("  <link href=\"{}\" rel=\"alternate\"/>\n", base_url));
    xml.push_str(&format!("  <id>{}/</id>\n", base_url));

    if let Some(first) = articles.first() {
        let updated = first.publish_at.unwrap_or(first.created_at);
        xml.push_str(&format!("  <updated>{}</updated>\n", updated.to_rfc3339()));
    }

    for article in &articles {
        let published = article.publish_at.unwrap_or(article.created_at);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", escape_xml(&article.title)));
        xml.push_str(&format!("    <link href=\"{}/articles/{}\"/>\n", base_url, article.slug));
        xml.push_str(&format!("    <id>{}/articles/{}</id>\n", base_url, article.slug));
        xml.push_str(&format!("    <published>{}</published>\n", published.to_rfc3339()));
        xml.push_str(&format!("    <updated>{}</updated>\n", article.updated_at.to_rfc3339()));
        if !article.short_text.is_empty() {
            xml.push_str(&format!("    <summary>{}</summary>\n", escape_xml(&article.short_text)));
        }
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        xml,
    ).into_response()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}
```

**Step 2: Declare module in api/mod.rs**

Add `pub mod feed;` to `backend/src/api/mod.rs`.

**Step 3: Register route in main.rs**

Add to `public_routes`:
```rust
.route("/feed.xml", get(api::feed::atom_feed))
```

**Step 4: Build and verify**

Run: `cd backend && cargo build`

**Step 5: Commit**

```bash
git add backend/src/api/feed.rs backend/src/api/mod.rs backend/src/main.rs
git commit -m "feat: add Atom RSS feed endpoint"
```

---

### Task 3.5: Update frontend types and API client

**Files:**
- Modify: `frontend/src/lib/api/types.ts`
- Modify: `frontend/src/lib/api/client.ts`

**Step 1: Update Article type**

Add to `Article` interface:
```typescript
cover_image_id: string | null;
reading_time_minutes: number;
```

Add to `Page` interface:
```typescript
template: 'default' | 'project';
```

**Step 2: Add related articles API call**

Add to `api` object (public section):
```typescript
getRelatedArticles: (slug: string) => fetchApi<Article[]>(`/articles/${slug}/related`),
```

**Step 3: Commit**

```bash
git add frontend/src/lib/api/types.ts frontend/src/lib/api/client.ts
git commit -m "feat: update frontend types for cover images, reading time, templates"
```

---

## Phase 4: Public Site Redesign

### Task 4.1: Remove sidebar, update public layout

**Files:**
- Modify: `frontend/src/routes/(public)/+layout.svelte`

**Step 1: Remove the sidebar from the site body**

Remove the `<aside class="site-sidebar desktop-only">...</aside>` block entirely.

Remove the `.site-sidebar` CSS rules.

Update `.site-body` to be centered single-column:
```css
.site-body {
  flex: 1;
  width: 100%;
  max-width: var(--width-wide);
  margin: 0 auto;
  padding: var(--space-xl) var(--space-lg);
}
```

Remove the duplicate nav in header that was there for desktop — keep just the header nav horizontal.

**Step 2: Update the header with bolder styling**

Update `.site-header` to remove the border and use a more subtle backdrop:
```css
.site-header {
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  position: sticky;
  top: 0;
  z-index: 50;
  backdrop-filter: blur(12px);
  background: rgba(255, 248, 240, 0.85);
}

[data-theme="dark"] .site-header {
  background: rgba(12, 10, 9, 0.85);
}
```

Update `.logo-title` to use accent gradient:
```css
.logo-title {
  font-family: var(--font-heading);
  font-size: 1.25rem;
  font-weight: 700;
  background: var(--gradient-accent);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
```

**Step 3: Redesign the footer**

```css
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
```

**Step 4: Verify build**

Run: `cd frontend && npm run check`

**Step 5: Commit**

```bash
git add frontend/src/routes/'(public)'/+layout.svelte
git commit -m "feat: remove sidebar, add glassmorphic header, redesign footer"
```

---

### Task 4.2: Redesign the homepage

**Files:**
- Modify: `frontend/src/routes/(public)/+page.svelte`
- Modify: `frontend/src/routes/(public)/+page.server.ts` (or `+page.ts`) — fetch featured apps + latest articles

**Step 1: Update the data loader to fetch both apps and articles for the homepage**

The homepage should always show: hero + featured projects + latest articles, regardless of `front_page_type`. Update the server load function to fetch both when front_page_type is the new `"homepage"` mode (or default to the new homepage layout).

**Step 2: Redesign the homepage template**

The homepage should have three sections:

1. **Hero section** — large display heading, tagline, gradient background, CTA buttons
2. **Featured projects** — glass cards showing 2-3 featured apps
3. **Latest articles** — horizontal scrolling card row

Use `InView` components for scroll-reveal on sections 2 and 3.

Key CSS for the hero:
```css
.hero {
  text-align: center;
  padding: var(--space-3xl) var(--space-lg);
  position: relative;
  overflow: hidden;
}

.hero::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-accent);
  opacity: 0.06;
  z-index: 0;
}

.hero-title {
  font-size: clamp(2.5rem, 5vw, 4.5rem);
  font-weight: 300;
  letter-spacing: -0.03em;
  line-height: 1.1;
  margin-bottom: var(--space-lg);
  position: relative;
  z-index: 1;
}

.hero-subtitle {
  font-size: clamp(1rem, 2vw, 1.25rem);
  color: var(--color-text-muted);
  max-width: 600px;
  margin: 0 auto var(--space-xl);
  line-height: 1.6;
  position: relative;
  z-index: 1;
}

.hero-actions {
  display: flex;
  gap: var(--space-md);
  justify-content: center;
  position: relative;
  z-index: 1;
}

.btn-gradient {
  background: var(--gradient-accent);
  color: white;
  padding: var(--space-md) var(--space-xl);
  border-radius: var(--radius-full);
  font-weight: 600;
  font-size: 1rem;
  transition: transform var(--transition-spring), box-shadow var(--transition-normal);
}

.btn-gradient:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(232, 146, 74, 0.3);
}
```

**Step 3: Style featured project cards with glassmorphism**

```css
.project-card {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(232, 222, 212, 0.5);
  border-radius: var(--radius-xl);
  padding: var(--space-xl);
  transition: transform var(--transition-spring), box-shadow var(--transition-normal);
}

[data-theme="dark"] .project-card {
  background: rgba(41, 37, 36, 0.6);
  border-color: rgba(61, 50, 41, 0.5);
}

.project-card:hover {
  transform: translateY(-4px) scale(1.01);
  box-shadow: var(--shadow-lg);
}
```

**Step 4: Commit**

```bash
git add frontend/src/routes/'(public)'/+page.svelte frontend/src/routes/'(public)'/+page.server.ts
git commit -m "feat: redesign homepage with hero, featured projects, latest articles"
```

---

### Task 4.3: Redesign article listing page

**Files:**
- Modify: `frontend/src/routes/(public)/articles/+page.svelte`

**Step 1: Change from vertical list to 2-column grid with rich cards**

Key changes:
- Grid layout: `grid-template-columns: repeat(auto-fill, minmax(340px, 1fr))`
- Cards include cover image (if present), category badge, title, excerpt, date, reading time
- Add category filter pills at the top (fetch categories from API)
- Wrap each card in `InView` for scroll-reveal with staggered delays

**Step 2: Style the article cards**

```css
.article-card {
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
  transition: transform var(--transition-spring), box-shadow var(--transition-normal);
  text-decoration: none;
  color: var(--color-text);
}

.article-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-lg);
}

.article-cover {
  width: 100%;
  aspect-ratio: 16/9;
  object-fit: cover;
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

.reading-time {
  display: flex;
  align-items: center;
  gap: 4px;
}
```

**Step 3: Commit**

```bash
git add frontend/src/routes/'(public)'/articles/+page.svelte
git commit -m "feat: redesign article listing with grid layout and rich cards"
```

---

### Task 4.4: Enhance article detail page

**Files:**
- Modify: `frontend/src/routes/(public)/articles/[slug]/+page.svelte`
- Modify: `frontend/src/routes/(public)/articles/[slug]/+page.server.ts` (or `+page.ts`)

**Step 1: Add cover image hero**

If the article has a `cover_image_id`, render a full-bleed hero image above the title.

**Step 2: Add reading time and progress bar**

Add a thin gradient bar at the top that fills as the user scrolls:
```svelte
<script>
  let progress = $state(0);
  function handleScroll() {
    const scrollable = document.documentElement.scrollHeight - window.innerHeight;
    progress = scrollable > 0 ? (window.scrollY / scrollable) * 100 : 0;
  }
</script>

<svelte:window on:scroll={handleScroll} />

<div class="progress-bar" style="width: {progress}%"></div>
```

```css
.progress-bar {
  position: fixed;
  top: 0;
  left: 0;
  height: 3px;
  background: var(--gradient-accent);
  z-index: 100;
  transition: width 50ms linear;
}
```

**Step 3: Add table of contents (auto-generated from headings)**

Parse the article HTML for h2/h3 headings on the server, pass them as `toc` data. Render as a sticky sidebar on desktop (>1024px), hidden on mobile.

**Step 4: Add related articles section**

Fetch related articles via `getRelatedArticles(slug)` and render at the bottom as a row of cards.

**Step 5: Add social sharing buttons**

Simple buttons for: Copy link (clipboard API), Twitter/X share URL, LinkedIn share URL.

**Step 6: Add SEO meta tags**

```svelte
<svelte:head>
  <title>{data.article.title}</title>
  <meta name="description" content={data.article.short_text} />
  <meta property="og:title" content={data.article.title} />
  <meta property="og:description" content={data.article.short_text} />
  <meta property="og:type" content="article" />
  {#if coverImageUrl}
    <meta property="og:image" content={coverImageUrl} />
  {/if}
</svelte:head>
```

**Step 7: Commit**

```bash
git add frontend/src/routes/'(public)'/articles/'[slug]'/
git commit -m "feat: enhance article page — cover image, progress bar, TOC, related articles, SEO"
```

---

### Task 4.5: Create project showcase page template

**Files:**
- Create: `frontend/src/lib/components/ProjectShowcase.svelte`
- Modify: `frontend/src/routes/(public)/[slug]/+page.svelte` (or wherever pages are rendered)

**Step 1: Create the ProjectShowcase component**

This component renders a page with `template: "project"` using the showcase layout:
- Hero banner with gradient background
- Content rendered in sections
- The page content is expected to contain structured HTML that the component parses for sections

Alternatively, since we already have the pages system, the project showcase could use the page content as free-form body, but styled differently with a dedicated layout.

**Step 2: Add showcase-specific styles**

Hero with gradient background, large padded sections, glassmorphism cards for feature highlights.

**Step 3: Commit**

```bash
git add frontend/src/lib/components/ProjectShowcase.svelte frontend/src/routes/'(public)'/
git commit -m "feat: add project showcase page template"
```

---

### Task 4.6: Build Cmd+K search modal

**Files:**
- Create: `frontend/src/lib/components/SearchModal.svelte`
- Modify: `frontend/src/routes/(public)/+layout.svelte` — mount and wire up keyboard shortcut

**Step 1: Create SearchModal component**

Key features:
- Opens on Cmd+K (Mac) or Ctrl+K (others)
- Full-screen overlay with centered modal
- Input auto-focused
- Debounced search (300ms) calling existing `api.search(q)`
- Results grouped by type (Articles, Projects, Pages)
- Keyboard navigation: arrow keys to select, Enter to navigate, Escape to close
- Highlighted matching text in results

**Step 2: Wire into public layout**

```svelte
<script>
  import SearchModal from '$lib/components/SearchModal.svelte';
  let searchOpen = $state(false);
</script>

<svelte:window onkeydown={(e) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    searchOpen = !searchOpen;
  }
}} />

<SearchModal bind:open={searchOpen} />
```

Replace the desktop SearchBar in header with a button that opens the modal.

**Step 3: Commit**

```bash
git add frontend/src/lib/components/SearchModal.svelte frontend/src/routes/'(public)'/+layout.svelte
git commit -m "feat: add Cmd+K search modal with live results and keyboard navigation"
```

---

## Phase 5: View Transitions & Animation Polish

### Task 5.1: Add Svelte view transitions

**Files:**
- Modify: `frontend/src/routes/(public)/+layout.svelte`

**Step 1: Enable view transitions**

SvelteKit supports the View Transitions API via the `onNavigate` hook:

```typescript
import { onNavigate } from '$app/navigation';

onNavigate((navigation) => {
  if (!document.startViewTransition) return;
  return new Promise((resolve) => {
    document.startViewTransition(async () => {
      resolve();
      await navigation.complete;
    });
  });
});
```

**Step 2: Add view transition CSS**

```css
@keyframes fade-in {
  from { opacity: 0; }
}
@keyframes fade-out {
  to { opacity: 0; }
}
@keyframes slide-from-right {
  from { transform: translateX(30px); }
}
@keyframes slide-to-left {
  to { transform: translateX(-30px); }
}

::view-transition-old(root) {
  animation: 200ms ease-out both fade-out, 200ms ease-out both slide-to-left;
}
::view-transition-new(root) {
  animation: 300ms ease-out both fade-in, 300ms ease-out both slide-from-right;
}
```

**Step 3: Commit**

```bash
git add frontend/src/routes/'(public)'/+layout.svelte
git commit -m "feat: add view transitions for smooth page navigation"
```

---

### Task 5.2: Add animated gradient background to hero

**Files:**
- Modify: `frontend/src/routes/(public)/+page.svelte`

**Step 1: Add CSS animated gradient mesh**

```css
.hero::before {
  content: '';
  position: absolute;
  inset: -50%;
  background: conic-gradient(
    from 0deg at 50% 50%,
    rgba(232, 146, 74, 0.08) 0deg,
    rgba(232, 93, 93, 0.06) 120deg,
    rgba(123, 166, 140, 0.05) 240deg,
    rgba(232, 146, 74, 0.08) 360deg
  );
  animation: rotate-gradient 20s linear infinite;
  z-index: 0;
}

@keyframes rotate-gradient {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
```

**Step 2: Commit**

```bash
git add frontend/src/routes/'(public)'/+page.svelte
git commit -m "feat: add animated gradient background to homepage hero"
```

---

## Phase 6: Editor Overhaul

### Task 6.1: Install additional TipTap extensions

**Files:**
- Modify: `frontend/package.json`

**Step 1: Install extensions**

```bash
cd frontend && npm install @tiptap/extension-character-count @tiptap/extension-floating-menu @tiptap/extension-bubble-menu @tiptap/extension-dropcursor @tiptap/extension-image
```

Note: Some extensions (floating menu, bubble menu) may already be included in starter-kit or available separately for TipTap 3. Check the actual TipTap 3 package availability and adjust accordingly.

**Step 2: Commit**

```bash
git add frontend/package.json frontend/package-lock.json
git commit -m "chore: install additional TipTap extensions"
```

---

### Task 6.2: Rewrite RichTextEditor with floating toolbar and slash commands

**Files:**
- Modify: `frontend/src/lib/components/RichTextEditor.svelte`

This is the biggest single task. Key changes:

**Step 1: Replace fixed toolbar with bubble menu**

Use TipTap's BubbleMenu extension — it shows a floating toolbar on text selection. Configure it to show: Bold, Italic, Link, H2, H3, Code, Blockquote.

**Step 2: Add slash command support**

Create a slash command extension using TipTap's Suggestion utility:
- Triggered by typing `/` at the beginning of a new line
- Shows a dropdown menu with: Heading 1, Heading 2, Heading 3, Bullet List, Ordered List, Blockquote, Code Block, Image, Horizontal Rule, Table
- Searchable: typing `/head` filters to headings
- Keyboard navigable: arrow up/down, Enter to select, Escape to dismiss

**Step 3: Add markdown shortcuts**

TipTap's StarterKit already includes some input rules (e.g., `# ` for H1). Ensure these are all enabled:
- `## ` → H2, `### ` → H3
- `- ` or `* ` → bullet list
- `1. ` → ordered list
- `> ` → blockquote
- ``` ``` ``` → code block
- `**text**` → bold, `*text*` → italic

These are mostly already provided by StarterKit. Verify and enable any missing ones.

**Step 4: Add drag-and-drop image upload**

Handle the `drop` event on the editor:
```typescript
editor.setOptions({
  editorProps: {
    handleDrop: (view, event) => {
      const files = event.dataTransfer?.files;
      if (files?.length) {
        event.preventDefault();
        for (const file of files) {
          if (file.type.startsWith('image/')) {
            uploadAndInsertImage(file, view, event);
          }
        }
        return true;
      }
      return false;
    }
  }
});
```

**Step 5: Add word count in editor footer**

Use `@tiptap/extension-character-count`:
```svelte
<div class="editor-footer">
  <span class="word-count">{editor?.storage.characterCount.words() ?? 0} words</span>
</div>
```

**Step 6: Add undo/redo buttons with keyboard hints**

```svelte
<div class="editor-toolbar-top">
  <button title="Undo (Ctrl+Z)" onclick={() => editor?.chain().focus().undo().run()}>
    Undo
  </button>
  <button title="Redo (Ctrl+Shift+Z)" onclick={() => editor?.chain().focus().redo().run()}>
    Redo
  </button>
</div>
```

**Step 7: Verify editor works**

Run: `cd frontend && npm run check`

**Step 8: Commit**

```bash
git add frontend/src/lib/components/RichTextEditor.svelte
git commit -m "feat: rewrite editor with floating toolbar, slash commands, drag-and-drop"
```

---

### Task 6.3: Add cover image picker to article editor

**Files:**
- Modify: `frontend/src/lib/components/ArticleEditor.svelte`

**Step 1: Add cover image state and UI**

Add a prominent cover image area at the top of the editor:
```svelte
<div class="cover-image-area" onclick={() => coverPickerOpen = true}>
  {#if coverImageUrl}
    <img src={coverImageUrl} alt="Cover" class="cover-preview" />
    <button class="cover-remove" onclick|stopPropagation={() => { coverImageId = null; coverImageUrl = null; }}>
      Remove
    </button>
  {:else}
    <div class="cover-placeholder">
      <span>Click to add cover image</span>
    </div>
  {/if}
</div>
```

**Step 2: Wire cover image into save payload**

Add `cover_image_id: coverImageId` to the save payload.

**Step 3: Replace `confirm()` calls with ConfirmDialog**

Replace `if (!confirm('Move this article to trash?'))` with the new `ConfirmDialog` component.

Replace `if (!confirm('Restore this revision?'))` similarly.

**Step 4: Replace inline alerts with toasts**

Replace `successMsg = 'Saved successfully.'` with `toasts.success('Saved successfully')`.
Replace `error = ...` with `toasts.error(...)`.

**Step 5: Add save state indicator**

Show a colored dot in the header: green = saved, orange = unsaved changes, gray = draft.

**Step 6: Commit**

```bash
git add frontend/src/lib/components/ArticleEditor.svelte
git commit -m "feat: add cover image picker, toasts, confirm dialogs to article editor"
```

---

## Phase 7: Syntax Highlighting

### Task 7.1: Add Shiki for code block highlighting

**Files:**
- Modify: `frontend/package.json`
- Create: `frontend/src/lib/highlight.ts`
- Modify: `frontend/src/routes/(public)/articles/[slug]/+page.svelte` (or server load)

**Step 1: Install Shiki**

```bash
cd frontend && npm install shiki
```

**Step 2: Create highlight utility**

```typescript
// frontend/src/lib/highlight.ts
import { codeToHtml } from 'shiki';

export async function highlightCodeBlocks(html: string): Promise<string> {
  // Find all <pre><code> blocks and replace with highlighted versions
  const codeBlockRegex = /<pre><code(?:\s+class="language-(\w+)")?>([^]*?)<\/code><\/pre>/g;

  let result = html;
  const matches = [...html.matchAll(codeBlockRegex)];

  for (const match of matches) {
    const lang = match[1] || 'text';
    const code = match[2]
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&amp;/g, '&')
      .replace(/&quot;/g, '"');

    try {
      const highlighted = await codeToHtml(code, {
        lang,
        themes: { light: 'github-light', dark: 'github-dark' },
      });
      result = result.replace(match[0], highlighted);
    } catch {
      // Leave unhighlighted on failure
    }
  }

  return result;
}
```

**Step 3: Apply highlighting in the article page server load**

In the article page's load function, process the content through `highlightCodeBlocks` before returning to the page.

**Step 4: Commit**

```bash
git add frontend/src/lib/highlight.ts frontend/package.json frontend/package-lock.json frontend/src/routes/'(public)'/articles/'[slug]'/
git commit -m "feat: add Shiki syntax highlighting for code blocks"
```

---

## Phase 8: Final Polish

### Task 8.1: Add RSS link to head and header

**Files:**
- Modify: `frontend/src/app.html` or `+layout.svelte`

**Step 1: Add RSS auto-discovery link**

```html
<link rel="alternate" type="application/atom+xml" title="Feed" href="/feed.xml" />
```

**Step 2: Add RSS icon/link in the footer**

**Step 3: Commit**

```bash
git add frontend/src/app.html frontend/src/routes/'(public)'/+layout.svelte
git commit -m "feat: add RSS auto-discovery and footer link"
```

---

### Task 8.2: Add admin Toast component to admin layout

**Files:**
- Modify: `frontend/src/routes/admin/+layout.svelte`

**Step 1: Import and mount Toast component**

```svelte
<script>
  import Toast from '$lib/components/Toast.svelte';
</script>

<!-- ... existing layout ... -->
<Toast />
```

**Step 2: Commit**

```bash
git add frontend/src/routes/admin/+layout.svelte
git commit -m "feat: mount toast notifications in admin layout"
```

---

### Task 8.3: Performance and cache headers

**Files:**
- Modify: `backend/src/main.rs`

**Step 1: Add cache control middleware for uploads**

Use tower-http's `SetResponseHeader` layer or a custom middleware to add `Cache-Control: public, max-age=31536000, immutable` to `/uploads/*` responses (since uploaded files are content-addressed by ID and never change).

**Step 2: Commit**

```bash
git add backend/src/main.rs
git commit -m "feat: add cache headers for uploaded media"
```

---

### Task 8.4: Final build verification

**Step 1: Verify backend builds**

Run: `cd backend && cargo build`
Expected: successful build

**Step 2: Verify frontend builds**

Run: `cd frontend && npm run build`
Expected: successful build

**Step 3: Verify Docker build**

Run: `docker build -t pawtal:dev .`
Expected: successful image build

---

## Summary

| Phase | Tasks | Focus |
|-------|-------|-------|
| 1 | 1.1–1.3 | Design system foundation (fonts, colors, tokens) |
| 2 | 2.1–2.3 | Reusable components (InView, Toast, ConfirmDialog) |
| 3 | 3.1–3.5 | Backend changes (migrations, endpoints, API types) |
| 4 | 4.1–4.6 | Public site redesign (layout, homepage, articles, search) |
| 5 | 5.1–5.2 | View transitions and animation polish |
| 6 | 6.1–6.3 | Editor overhaul (floating toolbar, slash commands, cover images) |
| 7 | 7.1 | Syntax highlighting |
| 8 | 8.1–8.4 | Final polish (RSS, toasts, cache, verification) |

**Total: ~24 tasks across 8 phases**
