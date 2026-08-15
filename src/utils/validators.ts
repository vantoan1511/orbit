/**
 * Client-side validation utilities for Kubernetes resource fields.
 */

/**
 * Validates a Kubernetes resource name according to RFC 1123 DNS subdomain rules.
 * - At most 253 characters
 * - Lowercase alphanumeric characters, '-' or '.'
 * - Must start and end with an alphanumeric character
 */
export function isValidK8sName(name: string): boolean {
  if (!name || typeof name !== 'string') return false
  const trimmed = name.trim()
  if (trimmed.length === 0 || trimmed.length > 253) return false
  const regex = /^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$/
  return regex.test(trimmed)
}

/**
 * Validates a Kubernetes DNS label (e.g. container name, label key suffix) according to RFC 1123.
 * - At most 63 characters
 * - Lowercase alphanumeric characters or '-'
 * - Must start and end with an alphanumeric character
 */
export function isValidK8sLabel(name: string): boolean {
  if (!name || typeof name !== 'string') return false
  const trimmed = name.trim()
  if (trimmed.length === 0 || trimmed.length > 63) return false
  const regex = /^[a-z0-9]([-a-z0-9]*[a-z0-9])?$/
  return regex.test(trimmed)
}

/**
 * Validates a network port number (1 - 65535).
 */
export function isValidPort(port: number | string): boolean {
  const num = typeof port === 'string' ? parseInt(port, 10) : port
  if (isNaN(num)) return false
  return Number.isInteger(num) && num >= 1 && num <= 65535
}

/**
 * Validates an Ingress host header / domain name (e.g., example.com, *.example.com).
 */
export function isValidHost(host: string): boolean {
  if (!host || typeof host !== 'string') return false
  const trimmed = host.trim()
  if (trimmed.length === 0 || trimmed.length > 253) return false
  const regex =
    /^(\*\.)?([a-zA-Z0-9]([-a-zA-Z0-9]*[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([-a-zA-Z0-9]*[a-zA-Z0-9])?$/
  return regex.test(trimmed)
}

/**
 * Validates an HTTP/Ingress URL path.
 * Must start with '/' or be empty.
 */
export function isValidPath(path: string): boolean {
  if (!path) return true
  return path.startsWith('/')
}

/**
 * Parses a rulesSummary item string (e.g. "app.example.com -> /api (svc:80)")
 * into { host: string; path: string }
 */
export function parseRuleSummary(ruleStr: string): { host: string; path: string } | null {
  if (!ruleStr) return null
  const arrowIdx = ruleStr.indexOf('->')
  if (arrowIdx === -1) return null
  const hostPart = ruleStr.substring(0, arrowIdx).trim()
  const remainder = ruleStr.substring(arrowIdx + 2).trim()
  const spaceIdx = remainder.indexOf(' ')
  const pathPart = spaceIdx !== -1 ? remainder.substring(0, spaceIdx).trim() : remainder
  return {
    host: hostPart === '*' ? '' : hostPart.toLowerCase(),
    path: (pathPart || '/').toLowerCase()
  }
}
