/**
 * Returns a human-readable relative time string for a given ISO date string.
 * e.g. "2 hours ago", "3 days ago"
 */
export function relativeTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);

  if (diffSecs < 60) return 'just now';
  if (diffSecs < 3600) {
    const mins = Math.floor(diffSecs / 60);
    return `${mins} minute${mins !== 1 ? 's' : ''} ago`;
  }
  if (diffSecs < 86400) {
    const hours = Math.floor(diffSecs / 3600);
    return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
  }
  if (diffSecs < 86400 * 7) {
    const days = Math.floor(diffSecs / 86400);
    return `${days} day${days !== 1 ? 's' : ''} ago`;
  }
  if (diffSecs < 86400 * 30) {
    const weeks = Math.floor(diffSecs / (86400 * 7));
    return `${weeks} week${weeks !== 1 ? 's' : ''} ago`;
  }

  return new Intl.DateTimeFormat('en-US', { year: 'numeric', month: 'short', day: 'numeric' }).format(date);
}

/**
 * Generates a URL-safe slug from a title string.
 */
export function slugify(title: string): string {
  return title
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '')
    .replace(/[\s_]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

/**
 * Formats file size bytes into a human-readable string.
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Formats a date to a locale-aware short datetime string.
 */
export function formatDate(isoString: string): string {
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(isoString));
}
