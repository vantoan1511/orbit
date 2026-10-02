export interface WorkloadContextBadge {
  badgeText: string
  subText?: string
}

/**
 * Formats workload kind, name, and namespace into a display badge structure.
 */
export function formatWorkloadContextBadge(
  kind?: string,
  name?: string,
  namespace?: string
): WorkloadContextBadge {
  if (!name) {
    return { badgeText: '' }
  }

  const prefix = kind ? `${kind}/` : ''
  return {
    badgeText: `${prefix}${name}`,
    subText: namespace || undefined
  }
}

/**
 * Re-measures and reinitializes VirtualScroller dimensions when parent container dimensions change.
 * Optionally scrolls to bottom if following is enabled.
 */
export function recalculateVirtualScroller(
  vs: { init?: () => void } | null | undefined,
  isFollowing: boolean,
  onScrollToBottom: () => void
): boolean {
  if (!vs || typeof vs.init !== 'function') {
    return false
  }

  vs.init()

  if (isFollowing) {
    onScrollToBottom()
  }

  return true
}

/**
 * Console container CSS classes supporting light mode and dark mode.
 */
export const CONSOLE_CONTAINER_CLASSES =
  'flex-1 min-h-0 bg-(--bg-card) dark:bg-[#090d16] border border-(--border) rounded-lg p-3 font-mono text-xs text-zinc-800 dark:text-zinc-200 leading-relaxed selection:bg-(--accent)/20 dark:selection:bg-(--accent)/30 overflow-hidden relative'

/**
 * Log row item CSS classes with light and dark line hover states.
 */
export const LOG_ROW_CLASSES =
  'flex gap-2 hover:bg-zinc-100 dark:hover:bg-white/5 py-0.5 rounded px-1 items-center whitespace-nowrap overflow-hidden'

/**
 * Pod / container tag classes with high contrast in both themes.
 */
export const LOG_ORIGIN_BADGE_CLASSES =
  'text-zinc-600 dark:text-zinc-400 font-semibold text-[11px] shrink-0 select-none'
