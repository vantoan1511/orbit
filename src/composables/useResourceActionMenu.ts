import type ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { ref } from 'vue'

/**
 * Encapsulates the action menu refs and event handlers shared by all resource tables.
 *
 * Provides:
 * - `actionMenu` ref wired to a `<ResourceActionMenu>` component instance
 * - `selectedActionRow` ref tracking which row the menu was opened for
 * - `toggleActionMenu` for the three-dots button click
 * - `onRowContextMenu` for the right-click row event
 */
export function useResourceActionMenu<T, R = T>(mapRow?: (row: T) => R) {
  const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
  const selectedActionRow = ref<R | null>(null)

  const toggleActionMenu = (event: Event, data: T) => {
    event.stopPropagation()
    selectedActionRow.value = (mapRow ? mapRow(data) : data) as unknown as R
    actionMenu.value?.toggle(event)
  }

  const onRowContextMenu = (event: { originalEvent: Event; data: T }) => {
    event.originalEvent?.stopPropagation()
    event.originalEvent?.preventDefault()
    selectedActionRow.value = (mapRow ? mapRow(event.data) : event.data) as unknown as R
    actionMenu.value?.show(event.originalEvent)
  }

  return { actionMenu, selectedActionRow, toggleActionMenu, onRowContextMenu }
}
