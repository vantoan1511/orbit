import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import { useDialog } from 'primevue/usedialog'
import { computed, toValue, type Ref, type MaybeRefOrGetter } from 'vue'
import { useRouter } from 'vue-router'
import { kubernetesService } from '@/services/kubernetesService'
import ScaleDialog from '@/components/shared/ScaleDialog.vue'
import CloneIngressDialog from '@/components/shared/CloneIngressDialog.vue'
import CloneDeploymentDialog from '@/components/shared/CloneDeploymentDialog.vue'

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

  const confirmAction = (
    message: string,
    header: string,
    acceptLabel: string,
    acceptCallback: () => Promise<void>,
    severity: 'primary' | 'danger' = 'danger'
  ) => {
    confirm.require({
      message,
      header,
      icon: 'pi pi-exclamation-triangle',
      rejectProps: {
        label: 'Cancel',
        severity: 'secondary',
        variant: 'text',
        size: 'small'
      },
      acceptProps: {
        label: acceptLabel,
        severity,
        size: 'small'
      },
      accept: acceptCallback
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
    if (!['PersistentVolume', 'PersistentVolumeClaim', 'Node'].includes(resourceKind)) {
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

    // Clone (Ingress only)
    if (resourceKind === 'Ingress') {
      items.push({
        label: 'Clone',
        icon: 'pi pi-copy',
        command: () => {
          const row = selectedActionRow.value
          if (!row) return

          const ingressRow = row as T & { hosts?: string }
          const sourceHosts = ingressRow.hosts
            ? ingressRow.hosts
                .split(',')
                .map((h: string) => h.trim())
                .filter(Boolean)
            : []

          dialog.open(CloneIngressDialog, {
            props: {
              header: 'Clone Ingress',
              style: {
                width: '380px'
              },
              modal: true
            },
            data: {
              sourceName: row.name,
              sourceNamespace: row.namespace || 'default',
              sourceHosts
            },
            onClose: async (options) => {
              const result = options?.data as
                { newName: string; newNamespace: string; newHosts: string[] } | undefined
              if (result?.newName) {
                try {
                  await kubernetesService.cloneIngress({
                    sourceNamespace: row.namespace || 'default',
                    sourceName: row.name,
                    newName: result.newName,
                    newNamespace: result.newNamespace,
                    newHosts: result.newHosts
                  })
                } catch (e) {
                  toast.add({
                    severity: 'error',
                    summary: 'Error',
                    detail: e instanceof Error ? e.message : 'Failed to clone Ingress',
                    life: 5000
                  })
                }
              }
            }
          })
        }
      })
    }

    // Clone (Deployment only)
    if (resourceKind === 'Deployment') {
      items.push({
        label: 'Clone',
        icon: 'pi pi-copy',
        command: () => {
          const row = selectedActionRow.value
          if (!row) return

          dialog.open(CloneDeploymentDialog, {
            props: {
              header: 'Clone Deployment',
              style: {
                width: '380px'
              },
              modal: true
            },
            data: {
              sourceName: row.name,
              sourceNamespace: row.namespace || 'default'
            },
            onClose: async (options) => {
              const result = options?.data as { newName: string; newNamespace: string } | undefined
              if (result?.newName) {
                try {
                  await kubernetesService.cloneDeployment({
                    sourceNamespace: row.namespace || 'default',
                    sourceName: row.name,
                    newName: result.newName,
                    newNamespace: result.newNamespace
                  })
                } catch (e) {
                  toast.add({
                    severity: 'error',
                    summary: 'Error',
                    detail: e instanceof Error ? e.message : 'Failed to clone Deployment',
                    life: 5000
                  })
                }
              }
            }
          })
        }
      })
    }

    // Redeploy
    if (['Deployment', 'StatefulSet', 'DaemonSet'].includes(resourceKind)) {
      items.push({
        label: 'Redeploy',
        icon: 'pi pi-refresh',
        command: () => {
          const row = selectedActionRow.value
          if (!row) return
          confirmAction(
            `Are you sure you want to redeploy ${resourceKind} "${row.name}"?`,
            'Confirm Redeploy',
            'Redeploy',
            async () => {
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
            },
            'primary'
          )
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
          confirmAction(
            `Are you sure you want to restart (delete) Pod "${row.name}"?`,
            'Confirm Restart',
            'Restart',
            async () => {
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
          )
        }
      })
    }

    // Edit
    if (resourceKind !== 'Event') {
      items.push({
        label: 'Edit',
        icon: 'pi pi-file-edit',
        command: () => {
          if (selectedActionRow.value) {
            router.push({
              name: 'edit-workload',
              params: {
                kind: resourceKind,
                namespace: selectedActionRow.value.namespace || 'default',
                name: selectedActionRow.value.name
              }
            })
          }
        }
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
          confirmAction(
            `Are you sure you want to ${deleteLabel.toLowerCase()} ${resourceKind} "${row.name}"?`,
            `Confirm ${deleteLabel}`,
            deleteLabel,
            async () => {
              try {
                // Note: This try/catch only handles IPC dispatch transport failures.
                // Kubernetes API errors are handled globally by App.vue listening to OrbitEvents.ErrorOccurred.
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
          )
        }
      })
    }

    return items.filter((item, index, arr) => {
      if (item.separator) {
        if (index === 0 || index === arr.length - 1 || arr[index - 1]?.separator) {
          return false
        }
      }
      return true
    })
  })

  return { actionMenuItems }
}
