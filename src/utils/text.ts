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

/**
 * Safely decodes a base64 encoded string, supporting UTF-8 content.
 */
export function decodeBase64(val: string): string {
  if (!val) return ''
  try {
    return decodeURIComponent(escape(atob(val)))
  } catch {
    try {
      return atob(val)
    } catch {
      return val
    }
  }
}

/**
 * Safely encodes a string into base64, supporting UTF-8 content.
 */
export function encodeBase64(val: string): string {
  if (!val) return ''
  try {
    return btoa(unescape(encodeURIComponent(val)))
  } catch {
    try {
      return btoa(val)
    } catch {
      return val
    }
  }
}
