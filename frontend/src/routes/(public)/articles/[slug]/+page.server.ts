import type { PageServerLoad } from './$types';
import { error } from '@sveltejs/kit';
import type { Article } from '$lib/api/types';

export const load: PageServerLoad = async ({ fetch, params }) => {
  const res = await fetch(`/api/articles/${params.slug}`);

  if (res.status === 404) {
    throw error(404, 'Article not found');
  }

  if (!res.ok) {
    throw error(res.status, 'Failed to load article');
  }

  const article: Article = await res.json();

  if (article.status !== 'published') {
    throw error(404, 'Article not found');
  }

  // Fetch related articles in parallel
  const relatedRes = await fetch(`/api/articles/${params.slug}/related`);
  const related: Article[] = relatedRes.ok ? await relatedRes.json() : [];

  // Extract headings for TOC
  const headingRegex = /<h([23])[^>]*>(.*?)<\/h\1>/gi;
  const toc: { level: number; text: string; id: string }[] = [];
  let match;
  while ((match = headingRegex.exec(article.content)) !== null) {
    const text = match[2].replace(/<[^>]*>/g, '');
    const id = text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    toc.push({ level: parseInt(match[1]), text, id });
  }

  // Inject IDs into headings for anchor links
  let processedContent = article.content;
  for (const heading of toc) {
    const regex = new RegExp(`(<h${heading.level})(>)`, 'i');
    processedContent = processedContent.replace(
      regex,
      `$1 id="${heading.id}"$2`
    );
  }

  return { article: { ...article, content: processedContent }, related, toc };
};
