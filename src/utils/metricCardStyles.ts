/**
 * Resolves standard CSS classes for generic (non-Kubernetes-kind) metric icon boxes.
 */
export function getGenericMetricIconClass(): string {
  return 'bg-(--accent-soft) text-(--accent)'
}

/**
 * Resolves standard CSS classes for generic metric card subtext indicators.
 */
export function getGenericMetricSubtextClass(): string {
  return 'text-(--accent) font-medium'
}

/**
 * Resolves standard CSS classes for generic progress bars and usage fills.
 */
export function getGenericProgressClass(): string {
  return 'bg-(--accent)'
}
