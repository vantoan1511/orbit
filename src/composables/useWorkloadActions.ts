import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { computed, toValue, type Ref, type MaybeRefOrGetter } from 'vue'
import { useRouter } from 'vue-router'

export function useWorkloadActions<T extends { name: string; namespace?: string }>(
  selectedActionRow: Ref<T | null>,
  drawerVisible: Ref<boolean>,
  selectedWorkload: Ref<T | null>,
  kind?: MaybeRefOrGetter<string>
) {
  const toast = useToast()
  const router = useRouter()

  const actionMenuItems = computed<MenuItem[]>(() => {
    const items: MenuItem[] = []
    const resourceKind = toValue(kind) || 'Deployment'

    // Logs - supported for Deployment, StatefulSet, DaemonSet, ReplicaSet, Job, Pod
    const supportsLogs = [
      'Deployment',
      'StatefulSet',
      'DaemonSet',
      'ReplicaSet',
      'Job',
      'Pod'
    ].includes(resourceKind)
    if (supportsLogs) {
      items.push({
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
                kind: resourceKind
              }
            })
          }
        }
      })
    }

    // View Details - supported for all except PersistentVolume and PersistentVolumeClaim
    if (!['PersistentVolume', 'PersistentVolumeClaim'].includes(resourceKind)) {
      items.push({
        label: 'View Details',
        icon: 'pi pi-info',
        command: () => {
          if (selectedActionRow.value) {
            if (selectedWorkload) {
              selectedWorkload.value = selectedActionRow.value
            }
            drawerVisible.value = true
          }
        }
      })
    }

    items.push({ separator: true })

    // Redeploy - Deployment, StatefulSet, DaemonSet
    if (['Deployment', 'StatefulSet', 'DaemonSet'].includes(resourceKind)) {
      items.push({
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
      })
    }

    // Restart - Pods
    if (resourceKind === 'Pod') {
      items.push({
        label: 'Restart',
        icon: 'pi pi-power-off',
        command: () => {
          toast.add({
            severity: 'info',
            summary: 'Restart Pod',
            detail: `Restart triggered for pod: ${selectedActionRow.value?.name}`,
            life: 3000
          })
        }
      })
    }

    // Edit - supported for all except Event
    if (resourceKind !== 'Event') {
      items.push({
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
      })
    }

    // Scale - Deployment, StatefulSet, ReplicaSet
    if (['Deployment', 'StatefulSet', 'ReplicaSet'].includes(resourceKind)) {
      items.push({
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
      })
    }

    items.push({ separator: true })

    // Delete / Terminate - supported for all except Event
    if (resourceKind !== 'Event') {
      const deleteLabel = resourceKind === 'Pod' ? 'Terminate' : 'Delete'
      const deleteSeverity = resourceKind === 'Pod' ? 'warn' : 'warn' // both warn but structured separately if needed
      items.push({
        label: deleteLabel,
        icon: 'pi pi-trash',
        class: 'text-red-400 hover:text-red-300',
        command: () => {
          toast.add({
            severity: deleteSeverity,
            summary: deleteLabel,
            detail: `${deleteLabel} triggered for ${selectedActionRow.value?.name}`,
            life: 3000
          })
        }
      })
    }

    return items
  })

  return {
    actionMenuItems
  }
}
