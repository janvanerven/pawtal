import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch }) => {
  // Fetch menus and public settings in parallel; gracefully degrade on failure
  const [mainMenuRes, footerMenuRes, settingsRes] = await Promise.allSettled([
    fetch('/api/menus/main'),
    fetch('/api/menus/footer'),
    fetch('/api/settings/public'),
  ]);

  const mainMenu = mainMenuRes.status === 'fulfilled' && mainMenuRes.value.ok
    ? await mainMenuRes.value.json()
    : { menu: { id: '', name: 'main' }, items: [] };

  const footerMenu = footerMenuRes.status === 'fulfilled' && footerMenuRes.value.ok
    ? await footerMenuRes.value.json()
    : { menu: { id: '', name: 'footer' }, items: [] };

  const settings: Record<string, string> = settingsRes.status === 'fulfilled' && settingsRes.value.ok
    ? await settingsRes.value.json()
    : {};

  return { mainMenu, footerMenu, settings };
};
