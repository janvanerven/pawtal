import type { PageServerLoad } from './$types';
import type { Page, Article, App, PaginatedResponse } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, url }) => {
  const settingsRes = await fetch('/api/settings/public');
  const settings: Record<string, string> = settingsRes.ok ? await settingsRes.json() : {};

  const frontPageType = settings.front_page_type || 'articles';

  if (frontPageType === 'page' && settings.front_page_slug) {
    const pageRes = await fetch(`/api/pages/${settings.front_page_slug}`);
    if (pageRes.ok) {
      const pageData: Page = await pageRes.json();
      return { type: 'page' as const, page: pageData, settings };
    }
    // Fall through to default homepage if page not found
  }

  if (frontPageType === 'app_catalogue') {
    const pageNum = url.searchParams.get('page') || '1';
    const appsRes = await fetch(`/api/apps?page=${pageNum}`);
    const apps: PaginatedResponse<App> = appsRes.ok ? await appsRes.json() : { data: [], total: 0, page: 1, per_page: 20 };
    return { type: 'apps' as const, apps, settings };
  }

  // Default: curated homepage with featured apps + latest articles
  const [articlesRes, appsRes] = await Promise.all([
    fetch('/api/articles?page=1&per_page=6'),
    fetch('/api/apps?page=1&per_page=3'),
  ]);

  const articles: PaginatedResponse<Article> = articlesRes.ok
    ? await articlesRes.json()
    : { data: [], total: 0, page: 1, per_page: 6 };

  const featuredApps: PaginatedResponse<App> = appsRes.ok
    ? await appsRes.json()
    : { data: [], total: 0, page: 1, per_page: 3 };

  return { type: 'articles' as const, articles, featuredApps, settings };
};
