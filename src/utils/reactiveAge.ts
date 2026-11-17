export type AgeTickCadence = 'none' | 'minute' | 'second'

/**
 * Determines the minimal ticker cadence required for a Kubernetes age string.
 * - `second`: `^\d+s$` -> 1s updates
 * - `minute`: `^\d+m$` -> 60s updates
 * - `none`: hours/days/unknown/empty -> zero reactive ticker updates
 */
export function resolveAgeTickCadence(age?: string): AgeTickCadence {
  if (!age) return 'none'
  const trimmed = age.trim()
  if (/^\d+s$/.test(trimmed)) return 'second'
  if (/^\d+m$/.test(trimmed)) return 'minute'
  return 'none'
}

/**
 * Parses a compact Kubernetes age string (e.g. `42s`, `5m`, `2h`, `3d`) into seconds.
 */
export function parseAgeToSeconds(ageStr?: string): number {
  if (!ageStr) return 0
  const match = ageStr.trim().match(/^(\d+)([smhd])$/)
  if (!match) return 0
  const val = parseInt(match[1] || '0', 10)
  const unit = match[2] || 's'
  switch (unit) {
    case 's':
      return val
    case 'm':
      return val * 60
    case 'h':
      return val * 3600
    case 'd':
      return val * 86400
    default:
      return 0
  }
}

/**
 * Formats a non-negative number of seconds into a compact Kubernetes age string.
 */
export function formatAgeSeconds(seconds: number): string {
  if (seconds < 0) return '0s'
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
  return `${Math.floor(seconds / 86400)}d`
}
