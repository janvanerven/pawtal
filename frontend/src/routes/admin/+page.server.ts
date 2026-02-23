import type { PageServerLoad } from './$types';
import type { AuditLogEntry } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch }) => {
  const [pagesRes, articlesRes, auditRes] = await Promise.allSettled([
    fetch('/api/admin/pages?page=1&per_page=100'),
    fetch('/api/admin/articles?page=1&per_page=100'),
    fetch('/api/admin/audit-log?page=1&per_page=5'),
  ]);

  let pageStats = { draft: 0, published: 0 };
  let articleStats = { draft: 0, published: 0 };
  let recentAudit: AuditLogEntry[] = [];

  if (pagesRes.status === 'fulfilled' && pagesRes.value.ok) {
    const data = await pagesRes.value.json();
    for (const p of data.data ?? []) {
      if (p.status === 'draft') pageStats.draft++;
      if (p.status === 'published') pageStats.published++;
    }
  }

  if (articlesRes.status === 'fulfilled' && articlesRes.value.ok) {
    const data = await articlesRes.value.json();
    for (const a of data.data ?? []) {
      if (a.status === 'draft') articleStats.draft++;
      if (a.status === 'published') articleStats.published++;
    }
  }

  if (auditRes.status === 'fulfilled' && auditRes.value.ok) {
    const data = await auditRes.value.json();
    recentAudit = (data.data ?? []) as AuditLogEntry[];
  }

  return { pageStats, articleStats, recentAudit };
};
