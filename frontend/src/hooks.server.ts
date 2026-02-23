import type { HandleFetch } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';

/**
 * Rewrites server-side fetch calls to /api/* so they reach the Axum backend
 * directly (port 8080) instead of looping back to SvelteKit (port 3000).
 *
 * Also forwards the browser's Cookie header so authenticated requests
 * (e.g. /api/admin/me) include the session token.
 */
export const handleFetch: HandleFetch = async ({ request, fetch, event }) => {
  const url = new URL(request.url);

  if (url.pathname.startsWith('/api/')) {
    const backendOrigin = env.BACKEND_ORIGIN ?? 'http://127.0.0.1:8080';
    const backendUrl = `${backendOrigin}${url.pathname}${url.search}`;
    const newRequest = new Request(backendUrl, request);

    // Forward cookies from the original browser request
    const cookie = event.request.headers.get('cookie');
    if (cookie) {
      newRequest.headers.set('cookie', cookie);
    }

    return fetch(newRequest);
  }

  return fetch(request);
};
