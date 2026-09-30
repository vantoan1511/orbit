import assert from 'node:assert/strict'
import test from 'node:test'
import { calculateDropIndex, useTabDragDrop } from '../useTabDragDrop.ts'

test('calculateDropIndex returns correct index when dragging right', () => {
  // Array [0, 1, 2, 3] -> move 0 to before 2 -> [1, 0, 2, 3] -> new index 1
  assert.equal(calculateDropIndex(0, 2, 'before'), 1)
  // Array [0, 1, 2, 3] -> move 0 to after 2 -> [1, 2, 0, 3] -> new index 2
  assert.equal(calculateDropIndex(0, 2, 'after'), 2)
  // Move 0 to before 1 -> [0, 1, 2, 3] -> new index 0 (no change)
  assert.equal(calculateDropIndex(0, 1, 'before'), 0)
  // Move 0 to after 1 -> [1, 0, 2, 3] -> new index 1
  assert.equal(calculateDropIndex(0, 1, 'after'), 1)
})

test('calculateDropIndex returns correct index when dragging left', () => {
  // Array [0, 1, 2, 3] -> move 3 to before 1 -> [0, 3, 1, 2] -> new index 1
  assert.equal(calculateDropIndex(3, 1, 'before'), 1)
  // Array [0, 1, 2, 3] -> move 3 to after 1 -> [0, 1, 3, 2] -> new index 2
  assert.equal(calculateDropIndex(3, 1, 'after'), 2)
  // Move 3 to before 2 -> [0, 1, 3, 2] -> new index 2
  assert.equal(calculateDropIndex(3, 2, 'before'), 2)
  // Move 3 to after 2 -> [0, 1, 2, 3] -> new index 3 (no change)
  assert.equal(calculateDropIndex(3, 2, 'after'), 3)
})

test('calculateDropIndex returns original index when fromIndex === hoverIndex', () => {
  assert.equal(calculateDropIndex(1, 1, 'before'), 1)
  assert.equal(calculateDropIndex(1, 1, 'after'), 1)
  assert.equal(calculateDropIndex(0, 0, 'before'), 0)
})

test('useTabDragDrop manages drag state transitions and invokes onReorder', () => {
  let reorderCall: { from: number; to: number } | null = null

  const dnd = useTabDragDrop({
    onReorder: (from, to) => {
      reorderCall = { from, to }
    }
  })

  assert.equal(dnd.draggedIndex.value, null)
  assert.equal(dnd.draggedTabId.value, null)
  assert.equal(dnd.dropTargetIndex.value, null)
  assert.equal(dnd.dropPosition.value, null)
  assert.equal(dnd.isDraggingTab('tab-1'), false)
  assert.equal(dnd.getDropIndicator(0), null)

  // Mock DragEvent
  const dataTransferData: Record<string, string> = {}
  const mockDataTransfer = {
    effectAllowed: 'none',
    dropEffect: 'none',
    setData: (type: string, val: string) => {
      dataTransferData[type] = val
    }
  }

  const dragStartEvent = {
    dataTransfer: mockDataTransfer
  } as unknown as DragEvent

  dnd.handleDragStart(1, 'tab-2', dragStartEvent)

  assert.equal(dnd.draggedIndex.value, 1)
  assert.equal(dnd.draggedTabId.value, 'tab-2')
  assert.equal(dnd.isDraggingTab('tab-2'), true)
  assert.equal(dnd.isDraggingTab('tab-1'), false)
  assert.equal(mockDataTransfer.effectAllowed, 'move')
  assert.equal(dataTransferData['text/plain'], 'tab-2')

  // Drag over target tab (mock midpoint calculation)
  // Target tab at x=100, width=100 (midpoint=150)
  // clientX=120 -> 'before'
  const mockTargetElementBefore = {
    getBoundingClientRect: () => ({ left: 100, width: 100 })
  }
  const dragOverBeforeEvent = {
    clientX: 120,
    currentTarget: mockTargetElementBefore,
    dataTransfer: mockDataTransfer,
    preventDefault: () => {}
  } as unknown as DragEvent

  dnd.handleDragOver(3, dragOverBeforeEvent)
  assert.equal(dnd.dropTargetIndex.value, 3)
  assert.equal(dnd.dropPosition.value, 'before')
  assert.equal(dnd.getDropIndicator(3), 'before')
  assert.equal(dnd.getDropIndicator(0), null)

  // clientX=180 -> 'after'
  const dragOverAfterEvent = {
    clientX: 180,
    currentTarget: mockTargetElementBefore,
    dataTransfer: mockDataTransfer,
    preventDefault: () => {}
  } as unknown as DragEvent

  dnd.handleDragOver(3, dragOverAfterEvent)
  assert.equal(dnd.dropTargetIndex.value, 3)
  assert.equal(dnd.dropPosition.value, 'after')
  assert.equal(dnd.getDropIndicator(3), 'after')

  // Drop at target
  const dropEvent = {
    preventDefault: () => {}
  } as unknown as DragEvent

  // Dragging 1 over 3 with 'after' -> calculateDropIndex(1, 3, 'after') -> 3
  dnd.handleDrop(3, dropEvent)
  assert.deepEqual(reorderCall, { from: 1, to: 3 })

  // State should be reset after drop
  assert.equal(dnd.draggedIndex.value, null)
  assert.equal(dnd.draggedTabId.value, null)
  assert.equal(dnd.dropTargetIndex.value, null)
  assert.equal(dnd.dropPosition.value, null)

  // DragEnd resets any active state
  dnd.handleDragStart(0, 'tab-1', dragStartEvent)
  assert.equal(dnd.draggedIndex.value, 0)
  dnd.handleDragEnd()
  assert.equal(dnd.draggedIndex.value, null)
  assert.equal(dnd.draggedTabId.value, null)
})
