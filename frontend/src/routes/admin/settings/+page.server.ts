import type { PageServerLoad } from './$types';
import type { Page } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch }) => {
  let settings: Record<string, string> = {};
  let pages: Page[] = [];

  try {
    const [settingsRes, pagesRes] = await Promise.allSettled([
      fetch('/api/admin/settings'),
      fetch('/api/admin/pages?page=1&per_page=100'),
    ]);
    if (settingsRes.status === 'fulfilled' && settingsRes.value.ok) {
      settings = await settingsRes.value.json();
    }
    if (pagesRes.status === 'fulfilled' && pagesRes.value.ok) {
      const data = await pagesRes.value.json();
      pages = (data.data ?? []) as Page[];
    }
  } catch {
    // Non-fatal
  }

  return { settings, pages };
};
