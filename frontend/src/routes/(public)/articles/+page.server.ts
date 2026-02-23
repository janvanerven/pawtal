import type { PageServerLoad } from './$types';
import type { Article, PaginatedResponse } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, url }) => {
  const pageNum = url.searchParams.get('page') || '1';

  const res = await fetch(`/api/articles?page=${pageNum}&per_page=10`);
  const articles: PaginatedResponse<Article> = res.ok
    ? await res.json()
    : { data: [], total: 0, page: 1, per_page: 10 };

  return { articles };
};
