import { onScopeDispose, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { useKubernetesStore } from '../stores/kubernetesStore.ts'

export type RegisterVisibleKindFn = (kind: string) => () => void

/**
 * Manages visibility lifecycle of a Kubernetes resource kind in the UI.
 * Registers the kind as visible with the store so that watch events can be flushed
 * immediately or at responsive frame cadence while active, and unregisters when unmounted.
 */
export function useVisibleResourceKind(
  kind: MaybeRefOrGetter<string | undefined>,
  registerFn?: RegisterVisibleKindFn
): void {
  let unregisterCurrent: (() => void) | null = null

  function cleanup(): void {
    if (unregisterCurrent) {
      unregisterCurrent()
      unregisterCurrent = null
    }
  }

  const register =
    registerFn ??
    ((k: string) => {
      const k8sStore = useKubernetesStore()
      return k8sStore.registerVisibleKind(k)
    })

  watch(
    () => toValue(kind),
    (newKind) => {
      cleanup()
      if (newKind && typeof newKind === 'string') {
        unregisterCurrent = register(newKind)
      }
    },
    { immediate: true, flush: 'sync' }
  )

  onScopeDispose(() => {
    cleanup()
  })
}
