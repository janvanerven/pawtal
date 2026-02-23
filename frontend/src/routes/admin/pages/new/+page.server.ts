import type { PageServerLoad } from './$types';
import type { Category } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch }) => {
  let categories: Category[] = [];
  try {
    const res = await fetch('/api/admin/categories');
    if (res.ok) categories = await res.json() as Category[];
  } catch {
    // categories are optional — editor works without them
  }
  return { categories };
};
