import type { OrbitEventMap, OrbitEventName } from '@/types/events'
import {
  events as neuEvents,
  extensions as neuExtensions,
  filesystem as neuFilesystem,
  init as neuInit
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
  events.on('engineConnected', (payload) => {
    if (payload.status === 'ready') {
      console.log('Orbit Engine connected!', payload.message)
    } else {
      console.log('Orbit Engine connected with error:', payload.message)
    }
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
  on<K extends OrbitEventName>(event: K, handler: (data: OrbitEventMap[K]) => void) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return neuEvents.on(event, (evt: any) => {
      const payload = evt?.detail as OrbitEventMap[K]
      handler(payload)
    })
  },
  off<K extends OrbitEventName>(event: K, handler: (data: OrbitEventMap[K]) => void) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return neuEvents.off(event, handler as any)
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

const CORE_ENGINE_ID = 'vantoan1511.orbit.core.engine'

/**
 * Safe wrapper for Orbit Core Engine extension
 */
export const coreEngine = {
  dispatch(event: string, data?: unknown) {
    return extensions.dispatch(CORE_ENGINE_ID, event, data)
  }
}
