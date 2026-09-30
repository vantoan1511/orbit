export interface LayoutRouteMeta {
  fullHeight?: boolean
  padding?: 'standard' | 'compact' | 'none'
}

/**
 * Route-specific default layout configuration.
 */
export const ROUTE_LAYOUT_CONFIG: Record<string, LayoutRouteMeta> = {
  '/logs': {
    fullHeight: true,
    padding: 'compact'
  }
}

/**
 * Resolves the layout metadata given a route path and any route-level meta overrides.
 */
export function resolveRouteLayoutMeta(path?: string, meta?: LayoutRouteMeta): LayoutRouteMeta {
  const pathConfig = path ? ROUTE_LAYOUT_CONFIG[path] : undefined
  return {
    ...pathConfig,
    ...meta
  }
}

/**
 * Returns the CSS class string for the main layout container.
 * Full-height views (such as Logs) use `overflow-hidden` so that internal scroll containers
 * (e.g. VirtualScroller) manage scrolling without triggering outer page scrollbars.
 */
export function getLayoutMainClass(meta?: LayoutRouteMeta): string {
  const isFullHeight = meta?.fullHeight === true
  return `flex-1 min-h-0 relative flex flex-col ${isFullHeight ? 'overflow-hidden' : 'overflow-y-auto'}`
}

/**
 * Returns the CSS class string for the content wrapper padding.
 * - 'compact': `p-4` (for full-height / dense console views like Logs)
 * - 'none': `p-0`
 * - 'standard' or undefined: `p-8` (default standard page padding)
 */
export function getLayoutContentPaddingClass(meta?: LayoutRouteMeta): string {
  const paddingType = meta?.padding
  const paddingClass = paddingType === 'compact' ? 'p-4' : paddingType === 'none' ? 'p-0' : 'p-8'
  return `flex-1 min-h-0 flex flex-col ${paddingClass}`
}
