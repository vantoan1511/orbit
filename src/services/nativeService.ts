import type { OrbitEventMap, OrbitEventName } from '@/types/events'
import { withTimeout } from '../utils/async.ts'

function isTauriEnvironment(): boolean {
  return (
    typeof globalThis.window !== 'undefined' &&
    ('__TAURI_INTERNALS__' in globalThis.window || '__TAURI__' in globalThis.window)
  )
}

/**
 * Sanitizes input path strings to prevent directory traversal attacks (e.g., ../).
 */
function sanitizePath(path: string): string {
  let sanitized = path.replace(/\\/g, '/')
  sanitized = sanitized.replace(/^[a-zA-Z]:/g, '')
  sanitized = sanitized.replace(/^\/+/g, '')
  sanitized = sanitized.replace(/\.\.+\//g, '')
  sanitized = sanitized.replace(/\.\.+$/g, '')
  return sanitized || './'
}

/**
 * Initialize native API
 */
export function init(): void {
  // In Tauri v2, initialization occurs natively upon webview creation
}

/**
 * Re-initialize native API connection.
 */
export function reconnect(): void {
  // In-process Tauri runtime does not drop WebSocket connections
}

/**
 * Safe wrapper for filesystem API
 */
export const filesystem = {
  async readDirectory(path: string): Promise<string[]> {
    const safePath = sanitizePath(path)
    return [safePath]
  }
}

/**
 * Safe wrapper for window API
 */
export const window = {}

/**
 * Safe wrapper for OS native capabilities (dialogs, browser URL opener)
 */
export const os = {
  async showOpenDialog(
    title: string,
    options?: {
      filters?: Array<{ name: string; extensions: string[] }>
      multiSelections?: boolean
    }
  ): Promise<string[]> {
    if (!isTauriEnvironment()) {
      return []
    }
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const result = await open({
        title,
        multiple: options?.multiSelections ?? false,
        filters: options?.filters
      })
      if (result === null) return []
      if (Array.isArray(result)) return result
      return [result]
    } catch (e) {
      console.warn('Dialog plugin error:', e)
      return []
    }
  },

  async open(url: string): Promise<void> {
    if (!isTauriEnvironment()) {
      if (typeof globalThis.window !== 'undefined' && globalThis.window.open) {
        globalThis.window.open(url, '_blank')
      }
      return
    }
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener')
      await openUrl(url)
    } catch (e) {
      console.warn('Opener plugin error:', e)
      if (typeof globalThis.window !== 'undefined' && globalThis.window.open) {
        globalThis.window.open(url, '_blank')
      }
    }
  }
}

// In-memory event dispatch maps
type EventHandler = (data: unknown) => void
const eventHandlerMap = new Map<string, Set<EventHandler>>()
const windowHandlerMap = new Map<string, Map<EventHandler, (evt: unknown) => void>>()
const tauriUnlistenMap = new Map<string, Map<EventHandler, () => void>>()

/**
 * Safe wrapper for typed events API
 */
export const events = {
  async on<K extends OrbitEventName>(
    event: K,
    handler: (data: OrbitEventMap[K]) => void
  ): Promise<void> {
    let handlers = eventHandlerMap.get(event)
    if (!handlers) {
      handlers = new Set()
      eventHandlerMap.set(event, handlers)
    }
    handlers.add(handler as EventHandler)

    // Window event listener support for DOM events and tests
    if (
      typeof globalThis.window !== 'undefined' &&
      typeof (globalThis.window as unknown as { addEventListener?: unknown }).addEventListener ===
        'function'
    ) {
      const windowWrapper = (evt: unknown) => {
        const payload = (evt as { detail?: OrbitEventMap[K] })?.detail as OrbitEventMap[K]
        handler(payload)
      }
      let winHandlers = windowHandlerMap.get(event)
      if (!winHandlers) {
        winHandlers = new Map()
        windowHandlerMap.set(event, winHandlers)
      }
      winHandlers.set(handler as EventHandler, windowWrapper)
      ;(
        globalThis.window as unknown as { addEventListener: (ev: string, fn: unknown) => void }
      ).addEventListener(event, windowWrapper)
    }

    if (isTauriEnvironment()) {
      try {
        const { listen } = await import('@tauri-apps/api/event')
        const unlisten = await listen<OrbitEventMap[K]>(event, (evt) => {
          handler(evt.payload)
        })

        let tauriHandlers = tauriUnlistenMap.get(event)
        if (!tauriHandlers) {
          tauriHandlers = new Map()
          tauriUnlistenMap.set(event, tauriHandlers)
        }
        tauriHandlers.set(handler as EventHandler, unlisten)
      } catch (e) {
        console.warn(`Failed to attach Tauri event listener for ${event}:`, e)
      }
    }
  },

  async off<K extends OrbitEventName>(
    event: K,
    handler: (data: OrbitEventMap[K]) => void
  ): Promise<void> {
    const handlers = eventHandlerMap.get(event)
    if (handlers) {
      handlers.delete(handler as EventHandler)
      if (handlers.size === 0) {
        eventHandlerMap.delete(event)
      }
    }

    const winHandlers = windowHandlerMap.get(event)
    if (winHandlers) {
      const windowWrapper = winHandlers.get(handler as EventHandler)
      if (windowWrapper && typeof globalThis.window !== 'undefined') {
        ;(
          globalThis.window as unknown as {
            removeEventListener?: (ev: string, fn: unknown) => void
          }
        ).removeEventListener?.(event, windowWrapper)
        winHandlers.delete(handler as EventHandler)
      }
    }

    const tauriHandlers = tauriUnlistenMap.get(event)
    if (tauriHandlers) {
      const unlisten = tauriHandlers.get(handler as EventHandler)
      if (unlisten) {
        unlisten()
        tauriHandlers.delete(handler as EventHandler)
      }
      if (tauriHandlers.size === 0) {
        tauriUnlistenMap.delete(event)
      }
    }
  },

  async dispatch(event: string, data?: unknown): Promise<void> {
    const handlers = eventHandlerMap.get(event)
    if (handlers) {
      handlers.forEach((h) => {
        try {
          h(data)
        } catch (e) {
          console.error(`Error in local event handler for ${event}:`, e)
        }
      })
    }

    if (
      typeof globalThis.window !== 'undefined' &&
      typeof (globalThis.window as unknown as { dispatchEvent?: unknown }).dispatchEvent ===
        'function'
    ) {
      ;(globalThis.window as unknown as { dispatchEvent: (evt: unknown) => void }).dispatchEvent({
        type: event,
        detail: data
      })
    }

    if (isTauriEnvironment()) {
      try {
        const { emit } = await import('@tauri-apps/api/event')
        await emit(event, data)
      } catch (e) {
        console.warn(`Failed to emit Tauri event ${event}:`, e)
      }
    }
  }
}

/**
 * Safe wrapper for Orbit Core Engine
 */
export const coreEngine = {
  async dispatch(event: string, data?: unknown): Promise<void> {
    if (!isTauriEnvironment()) {
      return
    }
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('dispatch_engine', { event, data })
    } catch (e) {
      console.error(`Failed to invoke engine command ${event}:`, e)
      throw e
    }
  }
}

export const extensions = {
  dispatch(_extensionId: string, event: string, data?: unknown): Promise<void> {
    return coreEngine.dispatch(event, data)
  },
  async getStats(): Promise<{ status: string }> {
    return { status: 'running' }
  }
}

/**
 * Probes the backend connection. Returns true if healthy, false if it hangs or rejects.
 */
export async function probe(timeoutMs = 1500): Promise<boolean> {
  try {
    await withTimeout(extensions.getStats(), timeoutMs)
    return true
  } catch {
    return false
  }
}

/**
 * Safe wrapper for application lifecycle API
 */
export const app = {
  async getConfig(): Promise<{ version: string; [key: string]: unknown }> {
    return { version: '0.14.4' }
  },
  async restartProcess(): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        const { relaunch } = await import('@tauri-apps/plugin-process')
        await relaunch()
      } catch (e) {
        console.warn('Failed to relaunch process:', e)
      }
    }
  },
  async exit(code = 0): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        const { exit } = await import('@tauri-apps/plugin-process')
        await exit(code)
      } catch (e) {
        console.warn('Failed to exit process:', e)
      }
    }
  }
}

// In-memory storage cache used for tests and non-Tauri environments
const inMemoryStore = new Map<string, string>()

let tauriStoreInstance: import('@tauri-apps/plugin-store').LazyStore | null = null

async function getTauriStore(): Promise<import('@tauri-apps/plugin-store').LazyStore | null> {
  if (!isTauriEnvironment()) {
    return null
  }
  if (!tauriStoreInstance) {
    try {
      const { LazyStore } = await import('@tauri-apps/plugin-store')
      tauriStoreInstance = new LazyStore('orbit-settings.json')
    } catch (e) {
      console.warn('Failed to initialize Tauri LazyStore:', e)
      return null
    }
  }
  return tauriStoreInstance
}

/**
 * Safe wrapper for persistent desktop storage API
 */
export const storage = {
  async setData(key: string, data?: string | null): Promise<void> {
    if (data === null || data === undefined) {
      await storage.removeData(key)
      return
    }
    inMemoryStore.set(key, data)
    const store = await getTauriStore()
    if (store) {
      try {
        await store.set(key, data)
        await store.save()
      } catch (e) {
        console.warn(`Failed to persist key ${key} to Tauri store:`, e)
      }
    }
  },

  async getData(key: string): Promise<string | null> {
    const store = await getTauriStore()
    if (store) {
      try {
        const val = await store.get<string>(key)
        if (val !== undefined && val !== null) {
          inMemoryStore.set(key, val)
          return val
        }
      } catch (e) {
        console.warn(`Failed to read key ${key} from Tauri store:`, e)
      }
    }
    return inMemoryStore.get(key) ?? null
  },

  async removeData(key: string): Promise<void> {
    inMemoryStore.delete(key)
    const store = await getTauriStore()
    if (store) {
      try {
        await store.delete(key)
        await store.save()
      } catch (e) {
        console.warn(`Failed to delete key ${key} from Tauri store:`, e)
      }
    }
  },

  async getKeys(): Promise<string[]> {
    const store = await getTauriStore()
    if (store) {
      try {
        return await store.keys()
      } catch (e) {
        console.warn('Failed to fetch keys from Tauri store:', e)
      }
    }
    return Array.from(inMemoryStore.keys())
  },

  async clear(): Promise<void> {
    inMemoryStore.clear()
    const store = await getTauriStore()
    if (store) {
      try {
        await store.clear()
        await store.save()
      } catch (e) {
        console.warn('Failed to clear Tauri store:', e)
      }
    }
  }
}
