import { init as neuInit, filesystem as neuFilesystem } from '@neutralinojs/lib'

/**
 * Sanitizes input path strings to prevent directory traversal attacks (e.g., ../).
 */
function sanitizePath(path: string): string {
  // Normalize backslashes to forward slashes
  let sanitized = path.replace(/\\/g, '/')

  // Remove drive letters (e.g. C:) to prevent absolute path access on Windows
  sanitized = sanitized.replace(/^[a-zA-Z]:/g, '')

  // Remove leading slashes to prevent absolute path access on Unix-like systems
  sanitized = sanitized.replace(/^\/+/g, '')

  // Remove directory traversal sequences (e.g. '../', '..')
  sanitized = sanitized.replace(/\.\.+\//g, '')
  sanitized = sanitized.replace(/\.\.+$/g, '')

  return sanitized || './'
}

/**
 * Initialize Neutralinojs native API
 */
export function init(): void {
  neuInit()
}

/**
 * Safe wrapper for Neutralino filesystem API
 */
export const filesystem = {
  readDirectory(path: string) {
    const safePath = sanitizePath(path)
    return neuFilesystem.readDirectory(safePath)
  }
}

/**
 * Safe wrapper for Neutralino window API
 */
export const window = {
  // Add sanitized window wrapper functions as needed
}

/**
 * Safe wrapper for Neutralino os API
 */
export const os = {
  // Add sanitized os wrapper functions as needed
}

/**
 * Safe wrapper for Neutralino events API
 */
export const events = {
  // Add events wrapper functions as needed
}

/**
 * Safe wrapper for Neutralino app API
 */
export const app = {
  // Add app wrapper functions as needed
}
