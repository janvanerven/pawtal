export interface User {
  id: string;
  external_id: string;
  email: string;
  display_name: string;
  role: 'admin' | 'editor';
  created_at: string;
  last_login: string | null;
}

export interface Page {
  id: string;
  title: string;
  slug: string;
  content: string;
  status: 'draft' | 'published' | 'scheduled' | 'trashed';
  publish_at: string | null;
  author_id: string;
  created_at: string;
  updated_at: string;
  trashed_at: string | null;
}

export interface PageRevision {
  id: string;
  page_id: string;
  title: string;
  content: string;
  author_id: string;
  created_at: string;
}

export interface Article {
  id: string;
  title: string;
  slug: string;
  short_text: string;
  content: string;
  status: 'draft' | 'published' | 'scheduled' | 'trashed';
  publish_at: string | null;
  author_id: string;
  created_at: string;
  updated_at: string;
  trashed_at: string | null;
}

export interface ArticleRevision {
  id: string;
  article_id: string;
  title: string;
  short_text: string;
  content: string;
  author_id: string;
  created_at: string;
}

export interface Category {
  id: string;
  name: string;
  slug: string;
}

export interface Media {
  id: string;
  filename: string;
  original_filename: string;
  mime_type: string;
  size_bytes: number;
  width: number | null;
  height: number | null;
  alt_text: string;
  is_icon: boolean;
  uploaded_by: string;
  created_at: string;
}

export interface App {
  id: string;
  name: string;
  description: string;
  icon_id: string | null;
  url: string | null;
  page_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Menu {
  id: string;
  name: string;
}

export interface MenuItem {
  id: string;
  menu_id: string;
  label: string;
  link_type: 'page' | 'article' | 'url' | 'app_catalogue';
  link_target: string;
  parent_id: string | null;
  sort_order: number;
}

export interface AuditLogEntry {
  id: string;
  user_id: string;
  action: string;
  entity_type: string;
  entity_id: string;
  details: string;
  created_at: string;
}

export interface SearchResult {
  result_type: 'page' | 'article' | 'app';
  id: string;
  title: string;
  slug: string;
  snippet: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
}

export interface MenuResponse {
  menu: Menu;
  items: MenuItem[];
}
