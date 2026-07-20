import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import { useDialog } from 'primevue/usedialog'
import { computed, toValue, type Ref, type MaybeRefOrGetter } from 'vue'
import { useRouter } from 'vue-router'
import { kubernetesService } from '@/services/kubernetesService'
import ScaleDialog from '@/components/shared/ScaleDialog.vue'

export interface WorkloadActionOptions<T> {
  kind?: MaybeRefOrGetter<string>
  onViewDetails?: (row: T) => void
}

export function useWorkloadActions<T extends { name: string; namespace?: string }>(
  selectedActionRow: Ref<T | null>,
  options: WorkloadActionOptions<T> = {}
) {
  const toast = useToast()
  const confirm = useConfirm()
  const dialog = useDialog()
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
        command: () => {
          const row = selectedActionRow.value
          if (!row) return
          confirm.require({
            message: `Are you sure you want to redeploy ${resourceKind} "${row.name}"?`,
            header: 'Confirm Redeploy',
            icon: 'pi pi-exclamation-triangle',
            rejectProps: {
              label: 'Cancel',
              severity: 'secondary',
              outlined: true
            },
            acceptProps: {
              label: 'Redeploy',
              severity: 'danger'
            },
            accept: async () => {
              try {
                await kubernetesService.redeployResource({
                  namespace: row.namespace || 'default',
                  kind: resourceKind,
                  name: row.name
                })
              } catch (e) {
                toast.add({
                  severity: 'error',
                  summary: 'Error',
                  detail: e instanceof Error ? e.message : 'Failed to redeploy',
                  life: 5000
                })
              }
            }
          })
        }
      })
    }

    // Restart
    if (resourceKind === 'Pod') {
      items.push({
        label: 'Restart',
        icon: 'pi pi-power-off',
        command: () => {
          const row = selectedActionRow.value
          if (!row) return
          confirm.require({
            message: `Are you sure you want to restart (delete) Pod "${row.name}"?`,
            header: 'Confirm Restart',
            icon: 'pi pi-exclamation-triangle',
            rejectProps: {
              label: 'Cancel',
              severity: 'secondary',
              outlined: true
            },
            acceptProps: {
              label: 'Restart',
              severity: 'danger'
            },
            accept: async () => {
              try {
                await kubernetesService.restartPod({
                  namespace: row.namespace || 'default',
                  name: row.name
                })
              } catch (e) {
                toast.add({
                  severity: 'error',
                  summary: 'Error',
                  detail: e instanceof Error ? e.message : 'Failed to restart pod',
                  life: 5000
                })
              }
            }
          })
        }
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
        command: () => {
          const row = selectedActionRow.value as (T & { replicas?: { desired?: number } }) | null
          if (!row) return

          let currentReplicas = 1
          if (row.replicas && typeof row.replicas.desired === 'number') {
            currentReplicas = row.replicas.desired
          }

          dialog.open(ScaleDialog, {
            props: {
              header: `Scale ${resourceKind}`,
              style: {
                width: '320px'
              },
              modal: true
            },
            data: {
              name: row.name,
              kind: resourceKind,
              currentReplicas
            },
            onClose: async (options) => {
              const newReplicas = options?.data
              if (typeof newReplicas === 'number') {
                try {
                  await kubernetesService.scaleResource({
                    namespace: row.namespace || 'default',
                    kind: resourceKind,
                    name: row.name,
                    replicas: newReplicas
                  })
                } catch (e) {
                  toast.add({
                    severity: 'error',
                    summary: 'Error',
                    detail: e instanceof Error ? e.message : 'Failed to scale',
                    life: 5000
                  })
                }
              }
            }
          })
        }
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
        command: () => {
          const row = selectedActionRow.value
          if (!row) return
          confirm.require({
            message: `Are you sure you want to ${deleteLabel.toLowerCase()} ${resourceKind} "${row.name}"?`,
            header: `Confirm ${deleteLabel}`,
            icon: 'pi pi-exclamation-triangle',
            rejectProps: {
              label: 'Cancel',
              severity: 'secondary',
              outlined: true
            },
            acceptProps: {
              label: deleteLabel,
              severity: 'danger'
            },
            accept: async () => {
              try {
                await kubernetesService.deleteResource({
                  namespace: row.namespace || 'default',
                  kind: resourceKind,
                  name: row.name
                })
              } catch (e) {
                toast.add({
                  severity: 'error',
                  summary: 'Error',
                  detail: e instanceof Error ? e.message : `Failed to ${deleteLabel.toLowerCase()}`,
                  life: 5000
                })
              }
            }
          })
        }
      })
    }

    return items
  })

  return { actionMenuItems }
}
