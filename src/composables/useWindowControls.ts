import { onMounted, onUnmounted, ref } from 'vue'
import { windowService } from '../services/nativeService.ts'

export function useWindowControls() {
  const isMaximized = ref(false)

  const checkMaximized = async () => {
    try {
      isMaximized.value = await windowService.isMaximized()
    } catch {
      isMaximized.value = false
    }
  }

  const minimize = async () => {
    try {
      await windowService.minimize()
    } catch (e) {
      console.warn('Failed to minimize window:', e)
    }
  }

  const toggleMaximize = async () => {
    try {
      await windowService.toggleMaximize()
      await checkMaximized()
    } catch (e) {
      console.warn('Failed to toggle maximize window:', e)
    }
  }

  const close = async () => {
    try {
      await windowService.close()
    } catch (e) {
      console.warn('Failed to close window:', e)
    }
  }

  const startDragging = async () => {
    try {
      await windowService.startDragging()
    } catch (e) {
      console.warn('Failed to start dragging window:', e)
    }
  }

  const handleResize = () => {
    void checkMaximized()
  }

  if (
    typeof globalThis.window !== 'undefined' &&
    typeof (globalThis.window as unknown as { addEventListener?: unknown }).addEventListener ===
      'function'
  ) {
    try {
      onMounted(() => {
        void checkMaximized()
        ;(
          globalThis.window as unknown as { addEventListener: (ev: string, fn: unknown) => void }
        ).addEventListener('resize', handleResize)
      })
      onUnmounted(() => {
        ;(
          globalThis.window as unknown as {
            removeEventListener?: (ev: string, fn: unknown) => void
          }
        ).removeEventListener?.('resize', handleResize)
      })
    } catch {
      // Not in Vue setup lifecycle scope (e.g. unit tests)
    }
  }

  return {
    isMaximized,
    minimize,
    toggleMaximize,
    close,
    startDragging,
    checkMaximized
  }
}
