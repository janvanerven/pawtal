import type { PageServerLoad } from './$types';
import type { SearchResult } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, url }) => {
  const query = url.searchParams.get('q') || '';

  if (!query.trim()) {
    return { query, results: [] as SearchResult[] };
  }

  const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
  const results: SearchResult[] = res.ok ? await res.json() : [];

  return { query, results };
};
