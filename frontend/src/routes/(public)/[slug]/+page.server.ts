import type { PageServerLoad } from './$types';
import { error } from '@sveltejs/kit';
import type { Page } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, params }) => {
  const res = await fetch(`/api/pages/${params.slug}`);

  if (res.status === 404) {
    throw error(404, 'Page not found');
  }

  if (!res.ok) {
    throw error(res.status, 'Failed to load page');
  }

  const pageData: Page = await res.json();

  // Only show published pages to public visitors
  if (pageData.status !== 'published') {
    throw error(404, 'Page not found');
  }

  return { page: pageData };
};
