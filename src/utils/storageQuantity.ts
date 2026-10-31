/**
 * Kubernetes Storage Quantity Parser and Formatters
 *
 * Implements Kubernetes canonical resource quantity parsing for storage quantities:
 * - Binary IEC units: Ki, Mi, Gi, Ti, Pi, Ei (and explicit *iB variants)
 * - Decimal SI units: m (milli), k/K, M, G, T, P, E (and explicit *B variants)
 * - Raw byte counts and scientific notation
 */

const GIB_BYTES = 1024 ** 3

const BINARY_SI_POWERS: Record<string, number> = {
  ki: 1024,
  kib: 1024,
  mi: 1024 ** 2,
  mib: 1024 ** 2,
  gi: 1024 ** 3,
  gib: 1024 ** 3,
  ti: 1024 ** 4,
  tib: 1024 ** 4,
  pi: 1024 ** 5,
  pib: 1024 ** 5,
  ei: 1024 ** 6,
  eib: 1024 ** 6
}

function resolveMultiplier(rawSuffix: string): number | null {
  if (!rawSuffix) return 1

  // Handle case-sensitive 'm' (milli) first
  if (rawSuffix === 'm') {
    return 1e-3
  }

  // Check binary IEC units (case-insensitive for *i and *ib)
  const lower = rawSuffix.toLowerCase()
  if (Object.prototype.hasOwnProperty.call(BINARY_SI_POWERS, lower)) {
    return BINARY_SI_POWERS[lower] ?? null
  }

  // Decimal SI units (K/k, M, G, T, P, E)
  switch (rawSuffix) {
    case 'k':
    case 'K':
    case 'kB':
    case 'KB':
    case 'kb':
      return 1e3
    case 'M':
    case 'MB':
    case 'mb':
      return 1e6
    case 'G':
    case 'GB':
    case 'gb':
      return 1e9
    case 'T':
    case 'TB':
    case 'tb':
      return 1e12
    case 'P':
    case 'PB':
    case 'pb':
      return 1e15
    case 'E':
    case 'EB':
    case 'eb':
      return 1e18
    default:
      return null
  }
}

/**
 * Parses a Kubernetes storage string into bytes.
 * Returns 0 if input is invalid, negative, or unparseable (never returns NaN).
 */
export function parseStorageBytes(value: string | null | undefined): number {
  if (!value || typeof value !== 'string') return 0
  const trimmed = value.trim()
  if (!trimmed || trimmed === '-') return 0

  const match = trimmed.match(/^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*([a-zA-Z]*)$/)
  if (!match || !match[1]) return 0

  const num = parseFloat(match[1])
  if (isNaN(num) || num <= 0) return 0

  const multiplier = resolveMultiplier(match[2] || '')
  if (multiplier === null) return 0

  return num * multiplier
}

/**
 * Parses a Kubernetes storage quantity string directly into GiB.
 * Returns 0 if input is invalid, negative, or unparseable.
 */
export function parseCapacityToGiB(value: string | null | undefined): number {
  const bytes = parseStorageBytes(value)
  if (bytes <= 0) return 0
  return bytes / GIB_BYTES
}

/**
 * Checks if a string is a valid positive Kubernetes storage quantity.
 */
export function isValidStorageQuantity(value: string | null | undefined): boolean {
  if (!value || typeof value !== 'string') return false
  const trimmed = value.trim()
  if (!trimmed || trimmed === '-') return false

  const match = trimmed.match(/^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*([a-zA-Z]*)$/)
  if (!match || !match[1]) return false

  const num = parseFloat(match[1])
  if (isNaN(num) || num <= 0) return false

  return resolveMultiplier(match[2] || '') !== null
}

/**
 * Formats a GiB capacity into a human-readable storage string (MiB, GiB, TiB).
 */
export function formatStorageGiB(gib: number): string {
  if (!gib || gib <= 0) {
    return '0.0 GiB'
  }
  if (gib < 1) {
    return `${Math.round(gib * 1024)} MiB`
  }
  if (gib >= 1024) {
    return `${(gib / 1024).toFixed(1)} TiB`
  }
  return `${gib.toFixed(1)} GiB`
}

export interface DonutCapacitySummary {
  value: string
  unit: 'Total GiB' | 'Total TiB'
}

/**
 * Formats total cluster capacity for donut chart center labels.
 */
export function formatDonutTotalCapacity(totalGiB: number): DonutCapacitySummary {
  if (!totalGiB || totalGiB <= 0 || isNaN(totalGiB)) {
    return { value: '0', unit: 'Total GiB' }
  }

  if (totalGiB >= 1024) {
    const tib = totalGiB / 1024
    const formatted = Number(tib.toFixed(1)).toString()
    return { value: formatted, unit: 'Total TiB' }
  }

  const formatted = Number(totalGiB.toFixed(1)).toString()
  return { value: formatted, unit: 'Total GiB' }
}
