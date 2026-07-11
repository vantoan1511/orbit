import {
  init as neuInit,
  filesystem as neuFilesystem,
  events as neuEvents,
  extensions as neuExtensions
} from '@neutralinojs/lib'

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

  // Register listener for core extension connection
  events.on('coreConnected', (evt: unknown) => {
    const detail = (evt as { detail?: unknown })?.detail
    console.log('Rust core extension reports connection:', detail)
  })
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
  on(event: string, handler: (evt: unknown) => void) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return neuEvents.on(event, handler as (evt: any) => void)
  },
  off(event: string, handler: (evt: unknown) => void) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return neuEvents.off(event, handler as (evt: any) => void)
  },
  dispatch(event: string, data?: unknown) {
    return neuEvents.dispatch(event, data)
  }
}

/**
 * Safe wrapper for Neutralino extensions API
 */
export const extensions = {
  dispatch(extensionId: string, event: string, data?: unknown) {
    return neuExtensions.dispatch(extensionId, event, data)
  }
}

/**
 * Safe wrapper for Neutralino app API
 */
export const app = {
  // Add app wrapper functions as needed
}
