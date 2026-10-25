export const ELEVATION_FREE_INSTALL_MODES = ['currentUser'] as const

export type InstallerExecutionLevel =
  'asInvoker' | 'highestAvailable' | 'requireAdministrator' | 'unknown'

export const EXECUTION_LEVEL_PATTERN = /<requestedExecutionLevel\s+[^>]*level=["']([^"']+)["']/i

const KNOWN_EXECUTION_LEVELS: Readonly<Record<string, InstallerExecutionLevel>> = Object.freeze({
  asInvoker: 'asInvoker',
  highestAvailable: 'highestAvailable',
  requireAdministrator: 'requireAdministrator'
})

/**
 * Parses the requestedExecutionLevel attribute from a Windows PE application manifest XML string.
 * Uses a lookup table to return the strongly-typed level or 'unknown'.
 */
export function parseInstallerExecutionLevel(manifestXml: string): InstallerExecutionLevel {
  const match = EXECUTION_LEVEL_PATTERN.exec(manifestXml)
  if (!match || !match[1]) {
    return 'unknown'
  }

  return KNOWN_EXECUTION_LEVELS[match[1]] ?? 'unknown'
}

/**
 * Scans raw binary PE executable bytes for an embedded application manifest XML
 * containing requestedExecutionLevel. Decodes using Latin-1 to preserve 1:1 byte-to-char mapping.
 */
export function extractManifestXml(exe: Uint8Array): string | null {
  const text = Buffer.from(exe.buffer, exe.byteOffset, exe.byteLength).toString('latin1')
  const levelIndex = text.search(EXECUTION_LEVEL_PATTERN)
  if (levelIndex === -1) {
    return null
  }

  const assemblyStart = text.lastIndexOf('<assembly', levelIndex)
  const assemblyEnd = text.indexOf('</assembly>', levelIndex)

  if (assemblyStart !== -1 && assemblyEnd !== -1 && assemblyEnd > assemblyStart) {
    return text.slice(assemblyStart, assemblyEnd + '</assembly>'.length)
  }

  const tagStart = text.lastIndexOf('<requestedExecutionLevel', levelIndex)
  const tagEnd = text.indexOf('>', levelIndex)
  if (tagStart !== -1 && tagEnd !== -1) {
    return text.slice(tagStart, tagEnd + 1)
  }

  return null
}
