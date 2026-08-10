import { onUnmounted, ref, type Ref } from 'vue'

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

  const initialWidth = storageKey
    ? parseInt(localStorage.getItem(storageKey) || `${defaultWidth}`, 10)
    : defaultWidth

  const width: Ref<number> = ref(
    isNaN(initialWidth) || initialWidth < minWidth ? defaultWidth : initialWidth
  )
  const isResizing = ref(false)
  let startLeft = 0
  let lastValidWidth = width.value

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
      localStorage.setItem(storageKey, width.value.toString())
    }
  }

  onUnmounted(() => {
    stopResize()
  })

  return {
    width,
    isResizing,
    startResize
  }
}
