import { codeToHtml } from 'shiki';

export async function highlightCodeBlocks(html: string): Promise<string> {
  const codeBlockRegex = /<pre><code(?:\s+class="language-(\w+)")?>([^]*?)<\/code><\/pre>/g;

  let result = html;
  const matches = [...html.matchAll(codeBlockRegex)];

  for (const match of matches) {
    const lang = match[1] || 'text';
    const code = match[2]
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&amp;/g, '&')
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'");

    try {
      const highlighted = await codeToHtml(code, {
        lang,
        themes: { light: 'github-light', dark: 'github-dark' },
      });
      result = result.replace(match[0], highlighted);
    } catch {
      // Leave unhighlighted on failure (unknown language, etc.)
    }
  }

  return result;
}
