import type { LayoutServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ fetch }) => {
  try {
    const res = await fetch('/api/admin/me');
    if (!res.ok) throw new Error('Not authenticated');
    const user = await res.json();
    return { user };
  } catch {
    throw redirect(302, '/api/auth/login');
  }
};
