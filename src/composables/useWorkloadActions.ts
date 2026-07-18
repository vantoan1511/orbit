import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { computed, type Ref } from 'vue'

export function useWorkloadActions<T extends { name: string }>(
  selectedActionRow: Ref<T | null>,
  drawerVisible: Ref<boolean>,
  selectedWorkload: Ref<T | null>
) {
  const toast = useToast()

  const actionMenuItems = computed<MenuItem[]>(() => [
    {
      label: 'View Details',
      icon: 'pi pi-info',
      command: () => {
        if (selectedActionRow.value) {
          selectedWorkload.value = selectedActionRow.value
          drawerVisible.value = true
        }
      }
    },
    {
      label: 'Edit (YAML)',
      icon: 'pi pi-pencil',
      command: () => {
        toast.add({
          severity: 'info',
          summary: 'Edit YAML',
          detail: `Edit YAML triggered for ${selectedActionRow.value?.name}`,
          life: 3000
        })
      }
    },
    {
      label: 'Scale',
      icon: 'pi pi-scale',
      command: () => {
        toast.add({
          severity: 'info',
          summary: 'Scale',
          detail: `Scale triggered for ${selectedActionRow.value?.name}`,
          life: 3000
        })
      }
    },
    {
      label: 'View Logs',
      icon: 'pi pi-scroll',
      class:
        'text-violet-400 hover:text-violet-300 font-semibold border border-violet-500/20 bg-violet-500/5',
      command: () => {
        toast.add({
          severity: 'info',
          summary: 'View Logs',
          detail: `View Logs triggered for ${selectedActionRow.value?.name}`,
          life: 3000
        })
      }
    },
    {
      label: 'Delete',
      icon: 'pi pi-bin',
      class: 'text-red-400 hover:text-red-300',
      command: () => {
        toast.add({
          severity: 'warn',
          summary: 'Delete',
          detail: `Delete triggered for ${selectedActionRow.value?.name}`,
          life: 3000
        })
      }
    }
  ])

  return {
    actionMenuItems
  }
}
