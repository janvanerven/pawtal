import type { PageServerLoad } from './$types';
import type { Category, Page } from '$lib/api/types';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, params }) => {
  const [pageRes, catsRes] = await Promise.allSettled([
    fetch(`/api/admin/pages/${params.id}`),
    fetch('/api/admin/categories'),
  ]);

  if (pageRes.status === 'rejected' || !pageRes.value.ok) {
    throw error(404, 'Page not found');
  }

  const page = await pageRes.value.json() as Page;
  let categories: Category[] = [];
  if (catsRes.status === 'fulfilled' && catsRes.value.ok) {
    categories = await catsRes.value.json() as Category[];
  }

  return { page, categories };
};
