<script lang="ts">
  import type { PageData } from './$types';
  import { api } from '$lib/api';

  let { data }: { data: PageData } = $props();

  // Initialize form from loaded settings
  let siteTitle = $state(data.settings['site_title'] ?? '');
  let frontPageType = $state<'page' | 'articles' | 'app_catalogue'>(
    (data.settings['front_page_type'] as 'page' | 'articles' | 'app_catalogue') ?? 'articles'
  );
  let frontPageId = $state(data.settings['front_page_id'] ?? '');
  let appsPerPage = $state(data.settings['apps_per_page'] ?? '20');
  let appCatalogueIntro = $state(data.settings['app_catalogue_intro'] ?? '');
  let darkModeDefault = $state(data.settings['dark_mode_default'] === 'true');

  let saving = $state(false);
  let error = $state('');
  let successMsg = $state('');

  async function saveSettings() {
    saving = true;
    error = '';
    successMsg = '';
    try {
      await api.admin.updateSettings({
        site_title: siteTitle,
        front_page_type: frontPageType,
        front_page_id: frontPageType === 'page' ? frontPageId : '',
        apps_per_page: appsPerPage,
        app_catalogue_intro: appCatalogueIntro,
        dark_mode_default: String(darkModeDefault),
      });
      successMsg = 'Settings saved.';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Save failed';
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <title>Settings — Pawtal CMS</title>
</svelte:head>

<div class="settings-page">
  <div class="page-header">
    <h1>Settings</h1>
  </div>

  {#if error}
    <div class="alert alert-error">{error}</div>
  {/if}
  {#if successMsg}
    <div class="alert alert-success">{successMsg}</div>
  {/if}

  <form class="settings-form card" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
    <div class="form-section">
      <h2>Site</h2>

      <div class="field">
        <label for="site-title">Site Title</label>
        <input id="site-title" type="text" bind:value={siteTitle} placeholder="My Pawtal Site" />
      </div>
    </div>

    <div class="form-section">
      <h2>Front Page</h2>

      <div class="field">
        <label for="front-page-type">Front Page Type</label>
        <select id="front-page-type" bind:value={frontPageType}>
          <option value="articles">Articles</option>
          <option value="page">Page</option>
          <option value="app_catalogue">App Catalogue</option>
        </select>
      </div>

      {#if frontPageType === 'page'}
        <div class="field">
          <label for="front-page">Front Page</label>
          <select id="front-page" bind:value={frontPageId}>
            <option value="">-- Select a page --</option>
            {#each data.pages as p (p.id)}
              {#if p.status === 'published'}
                <option value={p.id}>{p.title}</option>
              {/if}
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <div class="form-section">
      <h2>App Catalogue</h2>

      <div class="field">
        <label for="apps-per-page">Apps Per Page</label>
        <input
          id="apps-per-page"
          type="number"
          bind:value={appsPerPage}
          min="1"
          max="100"
          style="max-width: 120px"
        />
      </div>

      <div class="field">
        <label for="catalogue-intro">App Catalogue Intro</label>
        <textarea
          id="catalogue-intro"
          bind:value={appCatalogueIntro}
          placeholder="Welcome to our app catalogue..."
          rows="4"
        ></textarea>
      </div>
    </div>

    <div class="form-section">
      <h2>Appearance</h2>

      <div class="field">
        <label class="checkbox-field">
          <input type="checkbox" bind:checked={darkModeDefault} />
          <span>Dark mode by default</span>
        </label>
      </div>
    </div>

    <div class="form-actions">
      <button type="submit" class="btn btn-primary" disabled={saving}>
        {saving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </form>
</div>

<style>
  .settings-page { max-width: 640px; }
  .page-header { margin-bottom: var(--space-lg); }

  .alert { padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); margin-bottom: var(--space-md); font-size: 0.875rem; }
  .alert-error { background: #FFEBEE; color: var(--color-accent); }
  .alert-success { background: #E8F5E9; color: #2E7D32; }

  .settings-form { display: flex; flex-direction: column; gap: 0; padding: 0; overflow: hidden; }

  .form-section {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .form-section:last-child { border-bottom: none; }

  .form-section h2 {
    font-size: 0.8rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin-bottom: 0;
  }

  .field { display: flex; flex-direction: column; gap: var(--space-xs); }
  .field label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text);
  }

  .checkbox-field {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
  }
  .checkbox-field input[type="checkbox"] { width: auto; border: none; padding: 0; margin: 0; }

  .form-actions {
    padding: var(--space-lg);
    background: var(--color-bg);
  }
</style>
