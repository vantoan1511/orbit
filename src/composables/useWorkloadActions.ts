import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { computed, type Ref } from 'vue'
import { useRouter } from 'vue-router'

export function useWorkloadActions<T extends { name: string; namespace?: string }>(
  selectedActionRow: Ref<T | null>,
  drawerVisible: Ref<boolean>,
  selectedWorkload: Ref<T | null>,
  kind?: string
) {
  const toast = useToast()
  const router = useRouter()

  const actionMenuItems = computed<MenuItem[]>(() => [
    {
      label: 'View Logs',
      icon: 'pi pi-compass',
      command: () => {
        if (selectedActionRow.value) {
          drawerVisible.value = false // close details drawer
          router.push({
            name: 'logs',
            query: {
              namespace: selectedActionRow.value.namespace || 'default',
              workload: selectedActionRow.value.name,
              kind: kind || 'Deployment'
            }
          })
        }
      }
    },
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
      separator: true
    },
    {
      label: 'Redeploy',
      icon: 'pi pi-refresh',
      command: () => {
        toast.add({
          severity: 'info',
          summary: 'Redeploy',
          detail: `Redeploy triggered for ${selectedActionRow.value?.name}`,
          life: 3000
        })
      }
    },
    {
      label: 'Edit',
      icon: 'pi pi-file-edit',
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
      icon: 'pi pi-sliders-h',
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
      separator: true
    },
    {
      label: 'Delete',
      icon: 'pi pi-trash',
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
