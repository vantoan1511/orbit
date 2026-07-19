import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { computed, toValue, type Ref, type MaybeRefOrGetter } from 'vue'
import { useRouter } from 'vue-router'

export interface WorkloadActionOptions<T> {
  kind?: MaybeRefOrGetter<string>
  onViewDetails?: (row: T) => void
}

export function useWorkloadActions<T extends { name: string; namespace?: string }>(
  selectedActionRow: Ref<T | null>,
  options: WorkloadActionOptions<T> = {}
) {
  const toast = useToast()
  const router = useRouter()

  const showToast = (
    summary: string,
    actionName: string = summary,
    severity: 'info' | 'warn' = 'info'
  ) => {
    toast.add({
      severity,
      summary,
      detail: `${actionName} triggered for ${selectedActionRow.value?.name}`,
      life: 3000
    })
  }

  const actionMenuItems = computed<MenuItem[]>(() => {
    const items: MenuItem[] = []
    const resourceKind = toValue(options.kind) || 'Deployment'

    // Logs
    if (
      ['Deployment', 'StatefulSet', 'DaemonSet', 'ReplicaSet', 'Job', 'Pod'].includes(resourceKind)
    ) {
      items.push({
        label: 'View Logs',
        icon: 'pi pi-compass',
        command: () => {
          if (selectedActionRow.value) {
            router.push({
              name: 'logs',
              query: {
                namespace: selectedActionRow.value.namespace || 'default',
                workload: selectedActionRow.value.name,
                kind: resourceKind
              }
            })
          }
        }
      })
    }

    // View Details
    if (!['PersistentVolume', 'PersistentVolumeClaim'].includes(resourceKind)) {
      items.push({
        label: 'View Details',
        icon: 'pi pi-info',
        command: () => {
          if (selectedActionRow.value && options.onViewDetails) {
            options.onViewDetails(selectedActionRow.value)
          }
        }
      })
    }

    items.push({ separator: true })

    // Redeploy
    if (['Deployment', 'StatefulSet', 'DaemonSet'].includes(resourceKind)) {
      items.push({
        label: 'Redeploy',
        icon: 'pi pi-refresh',
        command: () => showToast('Redeploy')
      })
    }

    // Restart
    if (resourceKind === 'Pod') {
      items.push({
        label: 'Restart',
        icon: 'pi pi-power-off',
        command: () => showToast('Restart Pod', 'Restart')
      })
    }

    // Edit
    if (resourceKind !== 'Event') {
      items.push({
        label: 'Edit',
        icon: 'pi pi-file-edit',
        command: () => showToast('Edit YAML')
      })
    }

    // Scale
    if (['Deployment', 'StatefulSet', 'ReplicaSet'].includes(resourceKind)) {
      items.push({
        label: 'Scale',
        icon: 'pi pi-sliders-h',
        command: () => showToast('Scale')
      })
    }

    items.push({ separator: true })

    // Delete / Terminate
    if (resourceKind !== 'Event') {
      const deleteLabel = resourceKind === 'Pod' ? 'Terminate' : 'Delete'
      items.push({
        label: deleteLabel,
        icon: 'pi pi-trash',
        class: 'text-red-400 hover:text-red-300',
        command: () => showToast(deleteLabel, deleteLabel, 'warn')
      })
    }

    return items
  })

  return { actionMenuItems }
}
