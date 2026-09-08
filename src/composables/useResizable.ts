import { getCurrentInstance, onUnmounted, ref, type Ref } from 'vue'
import { storage } from '../services/nativeService.ts'
import { DEFAULT_STORAGE_TIMEOUT_MS, withTimeout } from '../utils/async.ts'

export interface UseResizableOptions {
  minWidth?: number
  maxWidth?: number
  defaultWidth?: number
  collapseThreshold?: number
  storageKey?: string
  onCollapse?: () => void
}

export function useResizable(options: UseResizableOptions = {}) {
  const minWidth = options.minWidth ?? 180
  const maxWidth = options.maxWidth ?? 600
  const defaultWidth = options.defaultWidth ?? 260
  const collapseThreshold = options.collapseThreshold ?? 90
  const storageKey = options.storageKey

  const width: Ref<number> = ref(defaultWidth)
  const isResizing = ref(false)
  let startLeft = 0
  let lastValidWidth = defaultWidth

  const loadStoredWidth = async () => {
    if (!storageKey) return
    try {
      let raw: string | null = null
      try {
        if (typeof window !== 'undefined' && !(window as unknown as { NL_PORT?: number }).NL_PORT) {
          throw new Error('Neutralino runtime not available')
        }
        raw = await withTimeout(storage.getData(storageKey), DEFAULT_STORAGE_TIMEOUT_MS)
      } catch {
        // Fallback check for legacy localStorage
        if (typeof localStorage !== 'undefined') {
          try {
            const legacy = localStorage.getItem(storageKey)
            if (legacy) {
              raw = legacy
              try {
                await storage.setData(storageKey, legacy)
                localStorage.removeItem(storageKey)
              } catch (migrationErr) {
                console.warn(
                  `Failed to write migrated width for ${storageKey} to native storage:`,
                  migrationErr
                )
              }
            }
          } catch {
            // localStorage not accessible
          }
        }
      }

      if (raw) {
        const parsed = parseInt(raw, 10)
        if (!isNaN(parsed) && parsed >= minWidth && parsed <= maxWidth) {
          width.value = parsed
          lastValidWidth = parsed
        }
      }
    } catch (e) {
      console.warn(`Failed to load stored width for ${storageKey}:`, e)
    }
  }

  if (storageKey) {
    void loadStoredWidth()
  }

  const startResize = (e: MouseEvent, targetElement?: HTMLElement | null) => {
    e.preventDefault()
    if (!targetElement) return
    isResizing.value = true
    startLeft = targetElement.getBoundingClientRect().left
    lastValidWidth = width.value > 0 ? width.value : defaultWidth

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', stopResize)
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
  }

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing.value) return
    const rawWidth = e.clientX - startLeft

    if (rawWidth < collapseThreshold) {
      width.value = 0
    } else {
      width.value = Math.max(minWidth, Math.min(maxWidth, rawWidth))
      lastValidWidth = width.value
    }
  }

  const stopResize = () => {
    if (!isResizing.value) return
    isResizing.value = false
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', stopResize)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''

    if (width.value < collapseThreshold) {
      width.value = lastValidWidth >= minWidth ? lastValidWidth : defaultWidth
      options.onCollapse?.()
    } else if (storageKey) {
      void storage.setData(storageKey, width.value.toString()).catch((err) => {
        console.warn(`Failed to persist width for ${storageKey}:`, err)
      })
    }
  }

  if (getCurrentInstance()) {
    onUnmounted(() => {
      stopResize()
    })
  }

  return {
    width,
    isResizing,
    startResize,
    loadStoredWidth
  }
}
