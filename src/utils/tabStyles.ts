/**
 * Resolves the CSS class string for the tab item container based on active state.
 * - Active: elevated surface tone bg-(--bg-card), high-contrast text-primary, font-medium,
 *   flat border-t-transparent (eliminating any blue accent lines), and seamless flush border-b-(--bg-card).
 * - Inactive: recessed bg-(--bg-sidebar)/70, text-muted-color, hover:bg-(--bg-hover)/60, hover:text-primary,
 *   and structural bottom border border-b-(--border).
 */
export function getTabContainerClass(isActive: boolean): string {
  const base =
    'group relative flex items-center gap-2 h-8 px-3 border-r border-(--border) text-xs cursor-pointer select-none transition-all duration-150 shrink-0 max-w-52 min-w-28'

  if (isActive) {
    return `${base} bg-(--bg-card) text-primary font-medium border-t border-t-transparent border-b border-b-(--bg-card)`
  }

  return `${base} bg-(--bg-sidebar)/70 text-muted-color hover:bg-(--bg-hover)/60 hover:text-primary border-t border-t-transparent border-b border-b-(--border)`
}

/**
 * Resolves the CSS class string for the tab close button based on active state.
 * - Active: persistently visible (opacity-100) for instant clickability without hover hunting.
 * - Inactive: hidden by default, visible on hover (opacity-0 group-hover:opacity-100).
 */
export function getTabCloseButtonClass(isActive: boolean): string {
  const base =
    'w-4! h-4! p-0! shrink-0 hover:bg-(--bg-hover)! text-muted-color hover:text-primary transition-opacity'

  if (isActive) {
    return `${base} opacity-100`
  }

  return `${base} opacity-0 group-hover:opacity-100`
}
