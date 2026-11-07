import {
  getCurrentInstance,
  onActivated,
  onDeactivated,
  onScopeDispose,
  toValue,
  watch,
  type MaybeRefOrGetter
} from 'vue'
import { useKubernetesStore } from '../stores/kubernetesStore.ts'

export type RegisterVisibleKindFn = (kind: string) => () => void

export interface UseVisibleResourceKindReturn {
  pause: () => void
  resume: () => void
}

/**
 * Manages visibility lifecycle of a Kubernetes resource kind in the UI.
 * Registers the kind as visible with the store so that watch events can be flushed
 * immediately or at responsive frame cadence while active, and unregisters when unmounted
 * or deactivated by <KeepAlive>.
 */
export function useVisibleResourceKind(
  kind: MaybeRefOrGetter<string | undefined>,
  registerFn?: RegisterVisibleKindFn
): UseVisibleResourceKindReturn {
  let unregisterCurrent: (() => void) | null = null
  let isPaused = false

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

  function resume(): void {
    if (!isPaused) return
    isPaused = false
    const currentKind = toValue(kind)
    if (currentKind && typeof currentKind === 'string' && !unregisterCurrent) {
      unregisterCurrent = register(currentKind)
    }
  }

  function pause(): void {
    if (isPaused) return
    isPaused = true
    cleanup()
  }

  watch(
    () => toValue(kind),
    (newKind) => {
      cleanup()
      if (!isPaused && newKind && typeof newKind === 'string') {
        unregisterCurrent = register(newKind)
      }
    },
    { immediate: true, flush: 'sync' }
  )

  if (getCurrentInstance()) {
    onActivated(() => {
      resume()
    })
    onDeactivated(() => {
      pause()
    })
  }

  onScopeDispose(() => {
    cleanup()
  })

  return {
    pause,
    resume
  }
}
