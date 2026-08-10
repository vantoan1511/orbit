/**
 * Highlights matches of query within text by wrapping them in HTML <mark> tags.
 */
export function highlightMatch(text: string, query: string): string {
  if (!query) return text
  const escapedQuery = query.trim().replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&')
  if (!escapedQuery) return text
  const regex = new RegExp(`(${escapedQuery})`, 'gi')
  return text.replace(
    regex,
    '<mark class="bg-zinc-200 dark:bg-zinc-700 text-surface-900 dark:text-surface-100 px-0.5">$1</mark>'
  )
}
