import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

export interface CreateResourceDialogData {
  initialNamespace?: string
  [key: string]: unknown
}

export interface UseCreateResourceDialogOptions {
  onSuccess?: () => void
  onError?: () => void
}

/**
 * Reusable composable that encapsulates PrimeVue dynamic dialog lifecycle,
 * namespace resolution, and asynchronous creation event listeners.
 */
export function useCreateResourceDialog<
  TData extends CreateResourceDialogData = CreateResourceDialogData
>(options?: UseCreateResourceDialogOptions) {
  const dialogRef = inject<
    | Ref<{
        data?: TData
        close: () => void
      }>
    | undefined
  >('dialogRef')

  const k8sStore = useKubernetesStore()

  const namespace = ref('default')
  const isCreating = ref(false)
  const submittedName = ref('')

  const namespaceOptions = computed(() => {
    const list = k8sStore.namespaceList.map((ns) => ns.name)
    if (list.length === 0) return ['default']
    return list
  })

  const handleCommandSucceeded = (payload: { message: string }) => {
    if (isCreating.value && submittedName.value && payload.message.includes(submittedName.value)) {
      isCreating.value = false
      submittedName.value = ''
      options?.onSuccess?.()
      dialogRef?.value?.close()
    }
  }

  const handleErrorOccurred = () => {
    if (isCreating.value) {
      isCreating.value = false
      submittedName.value = ''
      options?.onError?.()
    }
  }

  onMounted(() => {
    events.on(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
    events.on(OrbitEvents.ErrorOccurred, handleErrorOccurred)

    if (dialogRef?.value?.data?.initialNamespace) {
      namespace.value = dialogRef.value.data.initialNamespace
    } else if (namespaceOptions.value.length > 0) {
      namespace.value = namespaceOptions.value.includes('default')
        ? 'default'
        : (namespaceOptions.value[0] ?? 'default')
    }
  })

  onUnmounted(() => {
    events.off(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
    events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
  })

  const handleCancel = () => {
    dialogRef?.value?.close()
  }

  const submitCreation = (resourceName: string) => {
    isCreating.value = true
    submittedName.value = resourceName
  }

  const handleCreationFailure = () => {
    isCreating.value = false
    submittedName.value = ''
  }

  const isNameTaken = (name: string, list: Array<{ namespace?: string; name: string }>) => {
    const trimmed = name.trim().toLowerCase()
    if (!trimmed) return false
    return list.some(
      (item) => item.namespace === namespace.value && item.name.toLowerCase() === trimmed
    )
  }

  return {
    dialogRef,
    k8sStore,
    namespace,
    namespaceOptions,
    isCreating,
    submittedName,
    handleCancel,
    submitCreation,
    handleCreationFailure,
    isNameTaken
  }
}
