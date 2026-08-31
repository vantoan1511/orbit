import { kubernetesService } from '@/services/kubernetesService'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import ScaleDialog from '@/components/shared/ScaleDialog.vue'
import { useConfirm } from 'primevue/useconfirm'
import { useDialog } from 'primevue/usedialog'
import { useToast } from 'primevue/usetoast'
import { computed, toValue, type MaybeRefOrGetter, type Ref } from 'vue'

export interface BulkActionItem {
  label: string
  icon?: string
  severity?: 'secondary' | 'success' | 'info' | 'warn' | 'help' | 'danger' | 'contrast'
  variant?: 'text' | 'outlined' | 'link'
  class?: string
  command: () => void
}

export interface WorkloadBulkActionOptions {
  kind?: MaybeRefOrGetter<string>
  clearSelection?: () => void
}

export function useWorkloadBulkActions<T extends { name: string; namespace?: string }>(
  selectedItems: Ref<T[]>,
  options: WorkloadBulkActionOptions = {}
) {
  const toast = useToast()
  const confirm = useConfirm()
  const dialog = useDialog()

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

  const bulkActions = computed<BulkActionItem[]>(() => {
    const items: BulkActionItem[] = []
    const resourceKind = toValue(options.kind) || KUBERNETES_RESOURCE_KIND.Deployment
    const count = selectedItems.value.length

    if (count === 0) {
      return items
    }

    // Redeploy (Workloads)
    const redeployKinds: readonly string[] = [
      KUBERNETES_RESOURCE_KIND.Deployment,
      KUBERNETES_RESOURCE_KIND.StatefulSet,
      KUBERNETES_RESOURCE_KIND.DaemonSet
    ]
    if (redeployKinds.includes(resourceKind)) {
      items.push({
        label: 'Redeploy',
        icon: 'pi pi-refresh',
        severity: 'secondary',
        variant: 'text',
        command: () => {
          confirmAction(
            `Are you sure you want to redeploy ${count} selected ${resourceKind}(s)?`,
            'Confirm Bulk Redeploy',
            'Redeploy All',
            async () => {
              let successCount = 0
              let failCount = 0
              await Promise.allSettled(
                selectedItems.value.map(async (row) => {
                  try {
                    await kubernetesService.redeployResource({
                      namespace: row.namespace || 'default',
                      kind: resourceKind,
                      name: row.name
                    })
                    successCount++
                  } catch {
                    failCount++
                  }
                })
              )
              toast.add({
                severity: failCount === 0 ? 'success' : 'warn',
                summary: 'Bulk Redeploy',
                detail: `Redeployed ${successCount} ${resourceKind}(s)${failCount > 0 ? `, ${failCount} failed` : ''}.`,
                life: 5000
              })
              options.clearSelection?.()
            },
            'primary'
          )
        }
      })
    }

    // Restart (Pods)
    if (resourceKind === KUBERNETES_RESOURCE_KIND.Pod) {
      items.push({
        label: 'Restart',
        icon: 'pi pi-power-off',
        severity: 'secondary',
        variant: 'text',
        command: () => {
          confirmAction(
            `Are you sure you want to restart (delete) ${count} selected Pod(s)?`,
            'Confirm Bulk Restart',
            'Restart All',
            async () => {
              let successCount = 0
              let failCount = 0
              await Promise.allSettled(
                selectedItems.value.map(async (row) => {
                  try {
                    await kubernetesService.restartPod({
                      namespace: row.namespace || 'default',
                      name: row.name
                    })
                    successCount++
                  } catch {
                    failCount++
                  }
                })
              )
              toast.add({
                severity: failCount === 0 ? 'success' : 'warn',
                summary: 'Bulk Restart',
                detail: `Restarted ${successCount} Pod(s)${failCount > 0 ? `, ${failCount} failed` : ''}.`,
                life: 5000
              })
              options.clearSelection?.()
            }
          )
        }
      })
    }

    // Scale (Deployment, StatefulSet, ReplicaSet)
    const scaleKinds: readonly string[] = [
      KUBERNETES_RESOURCE_KIND.Deployment,
      KUBERNETES_RESOURCE_KIND.StatefulSet,
      KUBERNETES_RESOURCE_KIND.ReplicaSet
    ]
    if (scaleKinds.includes(resourceKind)) {
      items.push({
        label: 'Scale',
        icon: 'pi pi-sliders-h',
        severity: 'secondary',
        variant: 'text',
        command: () => {
          dialog.open(ScaleDialog, {
            props: {
              header: `Scale ${count} ${resourceKind}(s)`,
              style: {
                width: '320px'
              },
              modal: true
            },
            data: {
              name: `${count} selected items`,
              kind: resourceKind,
              currentReplicas: 1
            },
            onClose: async (dialogOptions) => {
              const newReplicas = dialogOptions?.data
              if (typeof newReplicas === 'number') {
                let successCount = 0
                let failCount = 0
                await Promise.allSettled(
                  selectedItems.value.map(async (row) => {
                    try {
                      await kubernetesService.scaleResource({
                        namespace: row.namespace || 'default',
                        kind: resourceKind,
                        name: row.name,
                        replicas: newReplicas
                      })
                      successCount++
                    } catch {
                      failCount++
                    }
                  })
                )
                toast.add({
                  severity: failCount === 0 ? 'success' : 'warn',
                  summary: 'Bulk Scale',
                  detail: `Scaled ${successCount} ${resourceKind}(s) to ${newReplicas} replicas${failCount > 0 ? `, ${failCount} failed` : ''}.`,
                  life: 5000
                })
                options.clearSelection?.()
              }
            }
          })
        }
      })
    }

    // Delete / Terminate
    if (resourceKind !== KUBERNETES_RESOURCE_KIND.Event) {
      const deleteLabel = resourceKind === KUBERNETES_RESOURCE_KIND.Pod ? 'Terminate' : 'Delete'
      items.push({
        label: deleteLabel,
        icon: 'pi pi-trash',
        severity: 'danger',
        variant: 'text',
        class: 'text-rose-500 hover:text-rose-600',
        command: () => {
          confirmAction(
            `Are you sure you want to ${deleteLabel.toLowerCase()} ${count} selected ${resourceKind}(s)?`,
            `Confirm Bulk ${deleteLabel}`,
            `${deleteLabel} All`,
            async () => {
              let successCount = 0
              let failCount = 0
              await Promise.allSettled(
                selectedItems.value.map(async (row) => {
                  try {
                    await kubernetesService.deleteResource({
                      namespace: row.namespace || 'default',
                      kind: resourceKind,
                      name: row.name
                    })
                    successCount++
                  } catch {
                    failCount++
                  }
                })
              )
              toast.add({
                severity: failCount === 0 ? 'success' : 'warn',
                summary: `Bulk ${deleteLabel}`,
                detail: `${deleteLabel}d ${successCount} ${resourceKind}(s)${failCount > 0 ? `, ${failCount} failed` : ''}.`,
                life: 5000
              })
              options.clearSelection?.()
            }
          )
        }
      })
    }

    return items
  })

  return { bulkActions }
}
