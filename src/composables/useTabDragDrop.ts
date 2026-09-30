import { ref, type Ref } from 'vue'

export type DropPosition = 'before' | 'after'

/**
 * Pure calculation helper to determine the final insertion index when moving an element
 * from `fromIndex` to a target `hoverIndex` given relative `position` ('before' | 'after').
 */
export function calculateDropIndex(
  fromIndex: number,
  hoverIndex: number,
  position: DropPosition
): number {
  if (fromIndex === hoverIndex) {
    return fromIndex
  }

  if (fromIndex < hoverIndex) {
    return position === 'before' ? hoverIndex - 1 : hoverIndex
  } else {
    return position === 'before' ? hoverIndex : hoverIndex + 1
  }
}

export interface TabDragDropOptions {
  onReorder: (fromIndex: number, toIndex: number) => void
}

export interface UseTabDragDropReturn {
  draggedIndex: Ref<number | null>
  draggedTabId: Ref<string | null>
  dropTargetIndex: Ref<number | null>
  dropPosition: Ref<DropPosition | null>
  isDraggingTab: (tabId: string) => boolean
  getDropIndicator: (index: number) => DropPosition | null
  handleDragStart: (index: number, tabId: string, event: DragEvent) => void
  handleDragOver: (index: number, event: DragEvent) => void
  handleDragLeave: (index: number, event: DragEvent) => void
  handleDrop: (index: number, event: DragEvent) => void
  handleDragEnd: () => void
}

export function useTabDragDrop(options: TabDragDropOptions): UseTabDragDropReturn {
  const draggedIndex = ref<number | null>(null)
  const draggedTabId = ref<string | null>(null)
  const dropTargetIndex = ref<number | null>(null)
  const dropPosition = ref<DropPosition | null>(null)

  function isDraggingTab(tabId: string): boolean {
    return draggedTabId.value === tabId
  }

  function getDropIndicator(index: number): DropPosition | null {
    if (dropTargetIndex.value === index) {
      return dropPosition.value
    }
    return null
  }

  function handleDragStart(index: number, tabId: string, event: DragEvent): void {
    draggedIndex.value = index
    draggedTabId.value = tabId
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
      event.dataTransfer.setData('text/plain', tabId)
    }
  }

  function handleDragOver(index: number, event: DragEvent): void {
    event.preventDefault()
    if (draggedIndex.value === null) {
      return
    }

    const currentTarget = event.currentTarget as HTMLElement | null
    if (currentTarget) {
      const rect = currentTarget.getBoundingClientRect()
      const midpoint = rect.left + rect.width / 2
      dropPosition.value = event.clientX < midpoint ? 'before' : 'after'
      dropTargetIndex.value = index
    }

    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move'
    }
  }

  function handleDragLeave(index: number, event: DragEvent): void {
    const currentTarget = event.currentTarget as HTMLElement | null
    const relatedTarget = event.relatedTarget as Node | null
    if (currentTarget && relatedTarget && currentTarget.contains(relatedTarget)) {
      return
    }
    if (dropTargetIndex.value === index) {
      dropTargetIndex.value = null
      dropPosition.value = null
    }
  }

  function handleDrop(index: number, event: DragEvent): void {
    event.preventDefault()
    if (draggedIndex.value === null || dropPosition.value === null) {
      handleDragEnd()
      return
    }

    const toIndex = calculateDropIndex(draggedIndex.value, index, dropPosition.value)
    const fromIndex = draggedIndex.value

    handleDragEnd()

    if (toIndex !== fromIndex) {
      options.onReorder(fromIndex, toIndex)
    }
  }

  function handleDragEnd(): void {
    draggedIndex.value = null
    draggedTabId.value = null
    dropTargetIndex.value = null
    dropPosition.value = null
  }

  return {
    draggedIndex,
    draggedTabId,
    dropTargetIndex,
    dropPosition,
    isDraggingTab,
    getDropIndicator,
    handleDragStart,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    handleDragEnd
  }
}
