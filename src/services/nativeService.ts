import { OrbitEvents, type OrbitEventMap, type OrbitEventName } from '@/types/events'
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

// rAF-based batch queue for resourceUpdated events only.
// Prevents multiple Vue reactivity cycles per animation frame.
const resourceUpdateQueue: Array<OrbitEventMap['resourceUpdated']> = []
let resourceUpdateRafPending = false

function flushResourceUpdateQueue() {
  resourceUpdateRafPending = false
  if (resourceUpdateQueue.length === 0) return
  const batch = resourceUpdateQueue.splice(0)
  const handlers = eventHandlerMap.get(OrbitEvents.ResourceUpdated)
  if (!handlers) return
  for (const [key, wrapper] of handlers) {
    if (key === '__interceptor__') continue
    for (const payload of batch) {
      wrapper({ detail: payload })
    }
  }
}

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

    if (event === (OrbitEvents.ResourceUpdated as K)) {
      let handlers = eventHandlerMap.get(event)
      if (!handlers) {
        handlers = new Map()
        eventHandlerMap.set(event, handlers)
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const interceptor = (evt: any) => {
          const payload = evt?.detail as OrbitEventMap['resourceUpdated']
          resourceUpdateQueue.push(payload)
          if (!resourceUpdateRafPending) {
            resourceUpdateRafPending = true
            requestAnimationFrame(flushResourceUpdateQueue)
          }
        }
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        handlers.set('__interceptor__', interceptor as any)
        neuEvents.on(event, interceptor)
      }
      handlers.set(handler, wrapper)
      return Promise.resolve()
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
    if (event === (OrbitEvents.ResourceUpdated as K)) {
      const handlers = eventHandlerMap.get(event)
      if (handlers) {
        handlers.delete(handler)
        const realHandlers = [...handlers.keys()].filter((k) => k !== '__interceptor__')
        if (realHandlers.length === 0) {
          const interceptor = handlers.get('__interceptor__')
          if (interceptor) {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            neuEvents.off(event, interceptor as any)
          }
          eventHandlerMap.delete(event)
        }
      }
      return Promise.resolve()
    }

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
  },
  restartProcess() {
    return neuApp.restartProcess()
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
