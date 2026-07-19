import type { OrbitEventMap, OrbitEventName } from '@/types/events'
import {
  app as neuApp,
  events as neuEvents,
  extensions as neuExtensions,
  filesystem as neuFilesystem,
  init as neuInit,
  os as neuOs,
  storage as neuStorage
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
  showOpenDialog(
    title: string,
    options?: {
      filters?: Array<{ name: string; extensions: string[] }>
      multiSelections?: boolean
    }
  ): Promise<string[]> {
    return neuOs.showOpenDialog(title, options)
  },
  open(url: string): Promise<void> {
    return neuOs.open(url)
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const eventHandlerMap = new Map<string, Map<any, any>>()

/**
 * Safe wrapper for Neutralino events API
 */
export const events = {
  on<K extends OrbitEventName>(event: K, handler: (data: OrbitEventMap[K]) => void) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const wrapper = (evt: any) => {
      const payload = evt?.detail as OrbitEventMap[K]
      handler(payload)
    }

    let handlers = eventHandlerMap.get(event)
    if (!handlers) {
      handlers = new Map()
      eventHandlerMap.set(event, handlers)
    }
    handlers.set(handler, wrapper)

    return neuEvents.on(event, wrapper)
  },
  off<K extends OrbitEventName>(event: K, handler: (data: OrbitEventMap[K]) => void) {
    const handlers = eventHandlerMap.get(event)
    if (handlers) {
      const wrapper = handlers.get(handler)
      if (wrapper) {
        handlers.delete(handler)
        if (handlers.size === 0) {
          eventHandlerMap.delete(event)
        }
        return neuEvents.off(event, wrapper)
      }
    }
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
  getConfig() {
    return neuApp.getConfig()
  }
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

/**
 * Safe wrapper for Neutralino storage API
 */
export const storage = {
  setData(key: string, data?: string | null) {
    return neuStorage.setData(key, data ?? null)
  },
  getData(key: string) {
    return neuStorage.getData(key)
  },
  getKeys() {
    return neuStorage.getKeys()
  }
}
