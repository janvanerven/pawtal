import type {
  User, Page, PageRevision, Article, ArticleRevision,
  Category, Media, App, MenuItem, Menu, AuditLogEntry,
  SearchResult, PaginatedResponse, MenuResponse
} from './types';

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

async function fetchApi<T>(path: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {};
  if (options?.body && typeof options.body === 'string') {
    headers['Content-Type'] = 'application/json';
  }

  const res = await fetch(`/api${path}`, { ...options, headers: { ...headers, ...options?.headers } });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, err.error || res.statusText);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  // Public
  getPage: (slug: string) => fetchApi<Page>(`/pages/${slug}`),
  listArticles: (page = 1, perPage = 20) => fetchApi<PaginatedResponse<Article>>(`/articles?page=${page}&per_page=${perPage}`),
  getArticle: (slug: string) => fetchApi<Article>(`/articles/${slug}`),
  listApps: (page = 1) => fetchApi<PaginatedResponse<App>>(`/apps?page=${page}`),
  getMenu: (name: string) => fetchApi<MenuResponse>(`/menus/${name}`),
  getPublicSettings: () => fetchApi<Record<string, string>>(`/settings/public`),
  search: (q: string, type?: string) =>
    fetchApi<SearchResult[]>(`/search?q=${encodeURIComponent(q)}${type ? `&type=${type}` : ''}`),

  // Admin
  admin: {
    me: () => fetchApi<User>('/admin/me'),

    // Pages
    listPages: (page = 1, status?: string) =>
      fetchApi<PaginatedResponse<Page>>(`/admin/pages?page=${page}${status ? `&status=${status}` : ''}`),
    getPage: (id: string) => fetchApi<Page>(`/admin/pages/${id}`),
    createPage: (data: Partial<Page> & { category_ids?: string[] }) =>
      fetchApi<Page>('/admin/pages', { method: 'POST', body: JSON.stringify(data) }),
    updatePage: (id: string, data: Partial<Page> & { category_ids?: string[] }) =>
      fetchApi<Page>(`/admin/pages/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
    deletePage: (id: string) =>
      fetchApi<{ ok: boolean }>(`/admin/pages/${id}`, { method: 'DELETE' }),
    publishPage: (id: string) =>
      fetchApi<Page>(`/admin/pages/${id}/publish`, { method: 'POST' }),
    restorePage: (id: string) =>
      fetchApi<Page>(`/admin/pages/${id}/restore`, { method: 'POST' }),
    getPageRevisions: (id: string) =>
      fetchApi<PageRevision[]>(`/admin/pages/${id}/revisions`),
    restorePageRevision: (pageId: string, revId: string) =>
      fetchApi<Page>(`/admin/pages/${pageId}/revisions/${revId}/restore`, { method: 'POST' }),

    // Articles
    listArticles: (page = 1, status?: string) =>
      fetchApi<PaginatedResponse<Article>>(`/admin/articles?page=${page}${status ? `&status=${status}` : ''}`),
    getArticle: (id: string) => fetchApi<Article>(`/admin/articles/${id}`),
    createArticle: (data: Partial<Article> & { category_ids?: string[] }) =>
      fetchApi<Article>('/admin/articles', { method: 'POST', body: JSON.stringify(data) }),
    updateArticle: (id: string, data: Partial<Article> & { category_ids?: string[] }) =>
      fetchApi<Article>(`/admin/articles/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
    deleteArticle: (id: string) =>
      fetchApi<{ ok: boolean }>(`/admin/articles/${id}`, { method: 'DELETE' }),
    publishArticle: (id: string) =>
      fetchApi<Article>(`/admin/articles/${id}/publish`, { method: 'POST' }),
    restoreArticle: (id: string) =>
      fetchApi<Article>(`/admin/articles/${id}/restore`, { method: 'POST' }),
    getArticleRevisions: (id: string) =>
      fetchApi<ArticleRevision[]>(`/admin/articles/${id}/revisions`),
    restoreArticleRevision: (articleId: string, revId: string) =>
      fetchApi<Article>(`/admin/articles/${articleId}/revisions/${revId}/restore`, { method: 'POST' }),

    // Media
    listMedia: (page = 1, filter?: string) =>
      fetchApi<PaginatedResponse<Media>>(`/admin/media?page=${page}${filter ? `&filter=${filter}` : ''}`),
    uploadMedia: async (file: File, isIcon = false): Promise<Media> => {
      const formData = new FormData();
      formData.append('file', file);
      formData.append('is_icon', isIcon.toString());
      const res = await fetch('/api/admin/media', { method: 'POST', body: formData });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.statusText }));
        throw new ApiError(res.status, err.error);
      }
      return res.json();
    },
    deleteMedia: (id: string) =>
      fetchApi<{ ok: boolean }>(`/admin/media/${id}`, { method: 'DELETE' }),

    // Apps
    listApps: (page = 1) => fetchApi<PaginatedResponse<App>>(`/admin/apps?page=${page}`),
    getApp: (id: string) => fetchApi<App>(`/admin/apps/${id}`),
    createApp: (data: Partial<App>) =>
      fetchApi<App>('/admin/apps', { method: 'POST', body: JSON.stringify(data) }),
    updateApp: (id: string, data: Partial<App>) =>
      fetchApi<App>(`/admin/apps/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
    deleteApp: (id: string) =>
      fetchApi<{ ok: boolean }>(`/admin/apps/${id}`, { method: 'DELETE' }),
    reorderApps: (ids: string[]) =>
      fetchApi<{ ok: boolean }>('/admin/apps/reorder', { method: 'PUT', body: JSON.stringify(ids) }),

    // Categories
    listCategories: () => fetchApi<Category[]>('/admin/categories'),
    createCategory: (data: { name: string; slug?: string }) =>
      fetchApi<Category>('/admin/categories', { method: 'POST', body: JSON.stringify(data) }),
    updateCategory: (id: string, data: { name?: string; slug?: string }) =>
      fetchApi<Category>(`/admin/categories/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
    deleteCategory: (id: string) =>
      fetchApi<{ ok: boolean }>(`/admin/categories/${id}`, { method: 'DELETE' }),

    // Menus
    getMenu: (name: string) => fetchApi<MenuResponse>(`/admin/menus/${name}`),
    updateMenu: (name: string, items: Partial<MenuItem>[]) =>
      fetchApi<{ ok: boolean }>(`/admin/menus/${name}`, { method: 'PUT', body: JSON.stringify({ items }) }),

    // Settings
    getSettings: () => fetchApi<Record<string, string>>('/admin/settings'),
    updateSettings: (settings: Record<string, string>) =>
      fetchApi<{ ok: boolean }>('/admin/settings', { method: 'PUT', body: JSON.stringify(settings) }),

    // Trash
    listTrash: () => fetchApi<{ pages: Page[]; articles: Article[] }>('/admin/trash'),
    emptyTrash: () => fetchApi<{ deleted_pages: number; deleted_articles: number }>('/admin/trash/empty', { method: 'POST' }),

    // Audit
    listAuditLog: (page = 1) => fetchApi<PaginatedResponse<AuditLogEntry>>(`/admin/audit-log?page=${page}`),

    // Users
    listUsers: () => fetchApi<User[]>('/admin/users'),
    updateUserRole: (id: string, role: string) =>
      fetchApi<User>(`/admin/users/${id}/role`, { method: 'PUT', body: JSON.stringify({ role }) }),

    // Search
    search: (q: string, type?: string) =>
      fetchApi<SearchResult[]>(`/admin/search?q=${encodeURIComponent(q)}${type ? `&type=${type}` : ''}`),
  },
};

export { ApiError };
