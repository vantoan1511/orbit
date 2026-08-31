import type { MenuItem } from 'primevue/menuitem'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import { useDialog } from 'primevue/usedialog'
import { computed, toValue, type Ref, type MaybeRefOrGetter } from 'vue'
import { useRouter } from 'vue-router'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { kubernetesService } from '@/services/kubernetesService'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import ScaleDialog from '@/components/shared/ScaleDialog.vue'
import CloneIngressDialog from '@/components/shared/CloneIngressDialog.vue'
import CloneDeploymentDialog from '@/components/shared/CloneDeploymentDialog.vue'
import PortForwardDialog from '@/components/shared/PortForwardDialog.vue'

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
  const k8sStore = useKubernetesStore()

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
    const resourceKind = toValue(options.kind) || KUBERNETES_RESOURCE_KIND.Deployment

    // Logs
    if (
      [
        KUBERNETES_RESOURCE_KIND.Deployment,
        KUBERNETES_RESOURCE_KIND.StatefulSet,
        KUBERNETES_RESOURCE_KIND.DaemonSet,
        KUBERNETES_RESOURCE_KIND.ReplicaSet,
        KUBERNETES_RESOURCE_KIND.Job,
        KUBERNETES_RESOURCE_KIND.Pod
      ].includes(resourceKind as any)
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
    if (
      ![
        KUBERNETES_RESOURCE_KIND.PersistentVolume,
        KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
        KUBERNETES_RESOURCE_KIND.Node
      ].includes(resourceKind as any)
    ) {
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

    // Port Forwarding
    if (
      [
        KUBERNETES_RESOURCE_KIND.Deployment,
        KUBERNETES_RESOURCE_KIND.Pod,
        KUBERNETES_RESOURCE_KIND.Service,
        KUBERNETES_RESOURCE_KIND.StatefulSet,
        KUBERNETES_RESOURCE_KIND.ReplicaSet
      ].includes(resourceKind as any)
    ) {
      items.push({
        label: 'Port Forwarding',
        icon: 'pi pi-arrow-right-arrow-left',
        command: () => {
          const row = selectedActionRow.value
          if (!row) return

          dialog.open(PortForwardDialog, {
            props: {
              header: 'Port Forwarding',
              style: {
                width: '380px'
              },
              modal: true
            },
            data: {
              sourceName: row.name,
              sourceNamespace: row.namespace || 'default',
              kind: resourceKind,
              availablePorts:
                resourceKind === KUBERNETES_RESOURCE_KIND.Service && 'portsList' in row
                  ? (row as { portsList?: Array<{ port: number }> }).portsList?.map(
                      (p) => p.port
                    ) || []
                  : []
            },
            onClose: async (options) => {
              const result = options?.data as { localPort: number; remotePort: number } | undefined
              if (result?.localPort && result?.remotePort) {
                try {
                  await kubernetesService.startPortForward({
                    namespace: row.namespace || 'default',
                    kind: resourceKind,
                    name: row.name,
                    localPort: result.localPort,
                    remotePort: result.remotePort
                  })
                  toast.add({
                    severity: 'success',
                    summary: 'Port Forward Started',
                    detail: `Forwarding ${result.localPort} -> ${result.remotePort} for ${row.name}`,
                    life: 5000
                  })
                } catch (e) {
                  toast.add({
                    severity: 'error',
                    summary: 'Error',
                    detail: e instanceof Error ? e.message : 'Failed to start port forwarding',
                    life: 5000
                  })
                }
              }
            }
          })
        }
      })

      const row = selectedActionRow.value
      if (row) {
        const targetNamespace = (row.namespace || 'default').toLowerCase()
        const targetKind = resourceKind.toLowerCase()
        const targetName = row.name.toLowerCase()

        const activeForwards = k8sStore.activePortForwards.filter(
          (f: any) =>
            f.namespace.toLowerCase() === targetNamespace &&
            f.kind.toLowerCase() === targetKind &&
            f.name.toLowerCase() === targetName
        )

        if (activeForwards.length > 0) {
          items.push({
            label: 'Stop Port Forwarding',
            icon: 'pi pi-stop-circle',
            class: 'text-warning hover:opacity-80',
            command: async () => {
              const currentRow = selectedActionRow.value
              if (!currentRow) return

              for (const f of activeForwards) {
                try {
                  await kubernetesService.stopPortForward({ id: f.id })
                  toast.add({
                    severity: 'info',
                    summary: 'Port Forward Stopped',
                    detail: `Stopped port forward for ${currentRow.name}`,
                    life: 3000
                  })
                } catch (e) {
                  toast.add({
                    severity: 'error',
                    summary: 'Error',
                    detail: e instanceof Error ? e.message : 'Failed to stop port forwarding',
                    life: 5000
                  })
                }
              }
            }
          })
        }
      }
    }

    items.push({ separator: true })

    // Clone (Ingress only)
    if (resourceKind === KUBERNETES_RESOURCE_KIND.Ingress) {
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
    if (resourceKind === KUBERNETES_RESOURCE_KIND.Deployment) {
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
    if (
      [
        KUBERNETES_RESOURCE_KIND.Deployment,
        KUBERNETES_RESOURCE_KIND.StatefulSet,
        KUBERNETES_RESOURCE_KIND.DaemonSet
      ].includes(resourceKind as any)
    ) {
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

    // Rollback
    if (resourceKind === KUBERNETES_RESOURCE_KIND.Deployment) {
      items.push({
        label: 'Rollback',
        icon: 'pi pi-history',
        command: () => {
          const row = selectedActionRow.value
          if (!row) return
          confirmAction(
            `Are you sure you want to rollback Deployment "${row.name}" to the previous revision?`,
            'Confirm Rollback',
            'Rollback',
            async () => {
              try {
                await kubernetesService.rollbackDeployment({
                  namespace: row.namespace || 'default',
                  name: row.name
                })
              } catch (e) {
                toast.add({
                  severity: 'error',
                  summary: 'Error',
                  detail: e instanceof Error ? e.message : 'Failed to rollback deployment',
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
    if (resourceKind === KUBERNETES_RESOURCE_KIND.Pod) {
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
    if (resourceKind !== KUBERNETES_RESOURCE_KIND.Event) {
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
    if (
      [
        KUBERNETES_RESOURCE_KIND.Deployment,
        KUBERNETES_RESOURCE_KIND.StatefulSet,
        KUBERNETES_RESOURCE_KIND.ReplicaSet
      ].includes(resourceKind as any)
    ) {
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
    if (resourceKind !== KUBERNETES_RESOURCE_KIND.Event) {
      const deleteLabel = resourceKind === KUBERNETES_RESOURCE_KIND.Pod ? 'Terminate' : 'Delete'
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
