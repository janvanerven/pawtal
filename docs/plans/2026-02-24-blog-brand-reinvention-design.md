# Pawtal Brand Reinvention — Design Document

**Date:** 2026-02-24
**Scope:** Full visual overhaul, new features, editor improvements, technical upgrades
**Approach:** Brand Reinvention (Approach B)

## Context

Pawtal is a self-hosted CMS (Rust/Axum backend, SvelteKit 5 frontend, SQLite) used as a product/project hub. The current design is functional but generic. This redesign aims to make it bold, playful, and memorable — with animated micro-interactions, a curated homepage, rich project showcases, and a significantly improved editor experience.

---

## 1. Design System Overhaul

### Typography
- **Headings:** Space Grotesk (geometric, modern, techy) — weights 300, 500, 700
- **Body:** DM Sans (clean, readable, slightly rounded) — weights 400, 500, 700
- **Display headings:** `clamp(2.5rem, 5vw, 4.5rem)` with tight letter-spacing (`-0.03em`)
- **Font weight for hierarchy:** 300 for large display, 700 for section heads, 400 for body

### Colors
- **Primary:** `#E8924A` (warm orange, kept from current)
- **Primary hover:** `#D47E3A`
- **Accent gradient:** `linear-gradient(135deg, #E8924A, #E85D5D)` for CTAs, badges, highlights
- **Light mode:** Cream background `#FFF8F0`, white surface
- **Dark mode (premium):**
  - Background: `#0C0A09` (near-black warm)
  - Surface: `#1C1917` (deep charcoal)
  - Elevated surface: `#292524` (for cards, modals)
  - Subtle warm tints instead of cold grays
- **Glassmorphism:** `backdrop-filter: blur(12px)` + semi-transparent surfaces in dark mode

### Layout
- Remove sidebar from public pages — navigation in header only
- Full-width sections with varied content widths (narrow for text, wide for grids)
- Asymmetric grids and editorial-style layouts
- Section dividers: angled cuts, gradient fades, or subtle SVG patterns
- Max content width: 1280px (wide sections), 720px (prose/reading)

### Motion & Animation
- **Page transitions:** Svelte 5 view transitions API (crossfade between routes)
- **Scroll-reveal:** IntersectionObserver-based fade/slide-in animations
- **Hover effects:** Spring physics on cards (scale 1.02 + shadow increase + subtle Y-axis rotation)
- **Background:** Animated gradient mesh or floating dots (CSS-only, performant)
- **Theme toggle:** Smooth color transitions (no flash)
- **Transition tokens:** `--transition-fast: 150ms ease`, `--transition-normal: 250ms ease`, `--transition-spring: 300ms cubic-bezier(0.34, 1.56, 0.64, 1)`

### Spacing & Radius
- Keep existing spacing scale (xs through 2xl)
- Slightly larger radii for cards: `--radius-lg: 20px`, `--radius-xl: 28px`

---

## 2. Public Site Redesign

### Homepage (Curated Storefront)
- **Hero section:**
  - Large display heading with site tagline
  - Animated gradient background (subtle, looping)
  - CTA buttons to projects and articles
  - Optional floating abstract shape or logo animation
- **Featured projects row:**
  - 2-3 hand-picked apps shown as rich glassmorphism cards
  - Icon, name, one-liner, screenshot/mockup thumbnail
  - Hover lift animation with spring physics
- **Latest articles row:**
  - Horizontal scrolling card row (3 visible, scroll for more)
  - Cards with cover image, title, date, excerpt
  - Subtle parallax effect on hover
- **Footer redesign:**
  - Dark background section
  - Multi-column layout: nav links, social links, "Built with Pawtal" badge

### Project/App Showcase Pages
- Powered by existing pages system with new `template: "project"` field
- **Layout:**
  - Hero banner with app icon, name, tagline (gradient background)
  - Screenshot gallery with lightbox (click to expand, swipe through)
  - Tech stack badges (inline pills)
  - Feature list with icons
  - Links to live app + source code
  - Related articles section (articles that share categories with this project)

### Article Pages (Reading Experience)
- **Cover/hero image:** Optional, full-width bleed above the title
- **Reading progress bar:** Thin accent-colored bar at top of viewport
- **Estimated reading time:** Shown next to the publish date
- **Table of contents:** Auto-generated from headings, sticky sidebar on desktop (hidden on mobile)
- **Code syntax highlighting:** Shiki integration (server-side processing preferred)
- **Related articles:** Bottom section, based on shared categories (3-4 cards)
- **Social sharing:** Copy link, Twitter/X, LinkedIn buttons
- **Back navigation:** Styled breadcrumb or back link

### Article Listing Page
- **Grid layout:** 2-column grid on desktop (instead of vertical list)
- **Rich cards:** Cover image, category badge, title, excerpt, date
- **Category filter pills:** Horizontal row at top, click to filter
- **Smooth filtering:** Crossfade animation when switching categories

### Search
- **Cmd+K modal overlay** (also accessible via search icon in header)
- Live results as you type
- Highlighted matching text in results
- Grouped results: Articles, Projects, Pages
- Keyboard navigation (arrow keys + Enter)

### RSS Feed
- New `/feed.xml` endpoint serving Atom/RSS XML
- Includes published articles with title, excerpt, date, link

---

## 3. Admin — Editor Overhaul

### Rich Text Editor
- **Floating bubble toolbar:** Appears on text selection (replaces fixed toolbar). Shows formatting options contextually.
- **Slash commands:** Type `/` for command menu: heading, list, image, code block, divider, table. Searchable, keyboard-navigable.
- **Markdown shortcuts:** `## ` → H2, `- ` → bullet, `> ` → blockquote, `` ``` `` → code block, `**text**` → bold, etc.
- **Drag-and-drop images:** Drop images into editor → auto-upload to media library → insert
- **Image controls:** Click image to resize, add caption, change alignment (left/center/full-width)
- **Link previews:** Pasting a URL offers inline preview card (title + description + favicon)
- **Table extension:** TipTap table with toolbar for add/remove rows/columns
- **Undo/redo buttons** with keyboard shortcut hints in tooltips
- **Word count** in editor footer (live updating)

### Article Editor Form
- **Cover image picker:** Prominent area at top of editor, click to select from media library
- **Slug auto-generation** with inline editing and URL preview
- **Category chips:** Tag-style multi-select instead of dropdown
- **Save state indicator:** Colored dot showing draft/saved/unsaved changes
- **Side-by-side preview:** Toggle to see rendered output alongside editor

### Admin UX Polish
- **Toast notifications:** Slide-in success/error messages (replace inline alerts)
- **Styled confirmation modals:** Replace `window.confirm()` with custom dialogs
- **Better empty states:** Illustrations + helpful CTAs when lists are empty

---

## 4. Technical Changes

### Frontend
- **New font packages:** `@fontsource/space-grotesk`, `@fontsource/dm-sans`
- **View transitions:** Svelte 5 view transitions API for route changes
- **InView component:** Reusable Svelte component wrapping IntersectionObserver
- **Image optimization:** Blur-up placeholders (tiny base64 → full image), lazy loading
- **Shiki:** Syntax highlighting for code blocks (SSR or client-side)
- **SEO:** `<meta>` tags, Open Graph (title, description, cover image), JSON-LD structured data for articles
- **Component library:** Extract reusable components (Modal, Toast, Skeleton, Card, Badge, Chip)
- **Performance:** Font subsetting, preload critical assets, CSS optimization
- **New TipTap extensions:** `@tiptap/extension-table`, `@tiptap/extension-character-count`, floating menu, slash commands

### Backend (Rust/Axum)
- **Migration:** Add `cover_image_id` (nullable FK to media) to articles table
- **Migration:** Add `reading_time_minutes` (integer) to articles table
- **Migration:** Add `template` field to pages table (TEXT, default `"default"`, options: `"default"` | `"project"`)
- **Reading time calculation:** On article create/update, compute from word count (~200 WPM)
- **Related articles endpoint:** `GET /api/articles/{slug}/related` — query by shared categories, exclude current, limit 4
- **RSS endpoint:** `GET /feed.xml` — Atom feed of published articles
- **Syntax highlighting:** Optional server-side processing with `syntect` crate on article save (store highlighted HTML)
- **Cover image in API responses:** Include cover image URL in article list/detail responses

### Infrastructure
- Add `Cache-Control` headers for static assets and uploaded media
- No other infrastructure changes needed — Docker setup is solid

---

## 5. What's NOT in Scope

- Commenting system
- Newsletter/email subscriptions
- Analytics dashboard
- User-facing accounts/profiles
- API for headless usage
- Scroll-jacking or 3D effects
- Command palette for admin (Cmd+K is search only)
