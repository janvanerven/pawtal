import type { PageServerLoad } from './$types';
import type { Category, Article } from '$lib/api/types';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, params }) => {
  const [articleRes, catsRes] = await Promise.allSettled([
    fetch(`/api/admin/articles/${params.id}`),
    fetch('/api/admin/categories'),
  ]);

  if (articleRes.status === 'rejected' || !articleRes.value.ok) {
    throw error(404, 'Article not found');
  }

  const article = await articleRes.value.json() as Article;
  let categories: Category[] = [];
  if (catsRes.status === 'fulfilled' && catsRes.value.ok) {
    categories = await catsRes.value.json() as Category[];
  }

  return { article, categories };
};
