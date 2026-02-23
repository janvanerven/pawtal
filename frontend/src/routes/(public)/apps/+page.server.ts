import type { PageServerLoad } from './$types';
import type { App, PaginatedResponse } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, url }) => {
  const pageNum = url.searchParams.get('page') || '1';

  const [appsRes, settingsRes] = await Promise.all([
    fetch(`/api/apps?page=${pageNum}`),
    fetch('/api/settings/public'),
  ]);

  const apps: PaginatedResponse<App> = appsRes.ok
    ? await appsRes.json()
    : { data: [], total: 0, page: 1, per_page: 20 };

  const settings: Record<string, string> = settingsRes.ok
    ? await settingsRes.json()
    : {};

  return { apps, settings };
};
