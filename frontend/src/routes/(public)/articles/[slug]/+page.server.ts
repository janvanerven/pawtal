import type { PageServerLoad } from './$types';
import { error } from '@sveltejs/kit';
import type { Article } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, params }) => {
  const res = await fetch(`/api/articles/${params.slug}`);

  if (res.status === 404) {
    throw error(404, 'Article not found');
  }

  if (!res.ok) {
    throw error(res.status, 'Failed to load article');
  }

  const article: Article = await res.json();

  if (article.status !== 'published') {
    throw error(404, 'Article not found');
  }

  return { article };
};
