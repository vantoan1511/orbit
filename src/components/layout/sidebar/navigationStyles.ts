/**
 * Resolves the styling classes for an Activity Bar icon button.
 */
export function getActivityBarItemClass(isActive: boolean, isDisabled: boolean): string {
  if (isActive) {
    return 'w-10 h-10 bg-(--accent-soft) text-(--accent) relative font-medium'
  }
  if (isDisabled) {
    return 'w-10 h-10 opacity-40 text-muted-color/50'
  }
  return 'w-10 h-10 text-muted-color hover:bg-(--bg-hover)'
}

/**
 * Resolves the styling classes for the Activity Bar active vertical indicator line.
 */
export function getActivityBarIndicatorClass(isActive: boolean): string {
  if (isActive) {
    return 'absolute left-0 top-1.5 bottom-1.5 w-0.5 rounded-r bg-(--accent) pointer-events-none'
  }
  return 'hidden'
}

/**
 * Resolves the styling classes for a navigation menu router-link item.
 */
export function getNavMenuItemClass(isActive: boolean): string {
  const base =
    'flex items-center gap-2.5 px-3 py-1.5 transition-colors text-sm cursor-pointer no-underline select-none my-0.5 mx-2 rounded-[var(--ui-radius-control)]'

  if (isActive) {
    return `${base} bg-(--accent-soft) text-(--accent) font-semibold`
  }
  return `${base} text-muted-color font-medium hover:bg-(--bg-hover)`
}
