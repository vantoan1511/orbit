/**
 * Formats a number with up to `maxDecimals` places, trimming trailing zeros and unneeded decimal points.
 * e.g. 300.254 (max 2) -> "300.25"
 * e.g. 300.200 (max 2) -> "300.2"
 * e.g. 300.000 (max 2) -> "300"
 */
export function formatDecimal(val: number, maxDecimals: number = 2): string {
  if (!Number.isFinite(val) || Math.abs(val) === 0) return '0'
  const str = val.toFixed(maxDecimals)
  const trimmed = str.includes('.') ? str.replace(/\.?0+$/, '') : str
  return trimmed === '-0' ? '0' : trimmed
}

/**
 * Formats a memory value in mebibytes (MiB) with up to 2 decimals and the 'Mi' unit suffix.
 * e.g. 300.25 -> "300.25Mi"
 * e.g. 300.0 -> "300Mi"
 */
export function formatMemoryMiB(mib: number): string {
  return `${formatDecimal(mib, 2)}Mi`
}

/**
 * Formats a CPU core quantity into either decimal cores or millicores ('m').
 * e.g. 2.25 -> "2.25 cores"
 * e.g. 0.25 -> "250m"
 * e.g. 0 -> "0m"
 */
export function formatCpuCores(cores: number): string {
  if (cores >= 1) {
    return `${formatDecimal(cores, 2)} cores`
  } else if (cores > 0) {
    return `${formatDecimal(cores * 1000, 2)}m`
  }
  return '0m'
}
