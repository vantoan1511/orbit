import { kubernetesService } from '../services/kubernetesService.ts'
import { events } from '../services/nativeService.ts'
import { OrbitEvents } from '../types/events.ts'
import { buildEditRequestId } from '../utils/editSession.ts'
import {
  computed,
  getCurrentInstance,
  onMounted,
  onUnmounted,
  ref,
  toValue,
  watch,
  type MaybeRefOrGetter
} from 'vue'
import * as yaml from 'yaml'

export interface UseResourceRawManifestOptions {
  kind: MaybeRefOrGetter<string>
  name: MaybeRefOrGetter<string | null | undefined>
  namespace?: MaybeRefOrGetter<string | null | undefined>
  visible?: MaybeRefOrGetter<boolean>
  fallbackResource?: MaybeRefOrGetter<Record<string, unknown> | null | undefined>
  onLoaded?: (manifest: Record<string, unknown>) => void
}

/**
 * Encapsulates asynchronous raw resource manifest fetching, YAML formatting,
 * event correlation, batch update reconciliation, and clipboard copying.
 */
export function useResourceRawManifest(options: UseResourceRawManifestOptions) {
  const rawManifest = ref<Record<string, unknown> | null>(null)
  const rawYamlData = ref<string | null>(null)
  const isYamlLoading = ref(false)
  const yamlError = ref<string | null>(null)
  const copied = ref(false)
  const activeRequestId = ref<string | null>(null)

  let copyTimeout: ReturnType<typeof setTimeout> | null = null

  const fetchRawYaml = async () => {
    const isVis = options.visible !== undefined ? toValue(options.visible) : true
    const kindVal = toValue(options.kind)
    const nameVal = toValue(options.name)
    const nsVal = toValue(options.namespace) || ''

    if (!isVis || !nameVal) return

    const reqId = buildEditRequestId(kindVal, nsVal, nameVal)
    activeRequestId.value = reqId
    isYamlLoading.value = true
    yamlError.value = null

    try {
      await kubernetesService.getResourceRaw({
        namespace: nsVal,
        kind: kindVal,
        name: nameVal,
        requestId: reqId
      })
    } catch (e) {
      if (activeRequestId.value === reqId) {
        isYamlLoading.value = false
        yamlError.value = String(e)
      }
    }
  }

  const handleRawData = (payload: {
    kind?: string
    name?: string
    namespace?: string
    data?: unknown
    requestId?: string
  }) => {
    const isVis = options.visible !== undefined ? toValue(options.visible) : true
    if (!isVis) return

    const currentKind = toValue(options.kind)
    const currentName = toValue(options.name)
    const currentNs = toValue(options.namespace) || ''
    if (!currentName) return

    if (payload?.requestId) {
      if (payload.requestId !== activeRequestId.value) return
    } else {
      if (payload.kind && payload.kind.toLowerCase() !== currentKind.toLowerCase()) return
      if (payload.name !== currentName) return
      if (payload.namespace !== undefined && payload.namespace !== currentNs) return
    }

    if (payload.data) {
      if (typeof payload.data === 'string') {
        try {
          rawManifest.value = JSON.parse(payload.data) as Record<string, unknown>
          rawYamlData.value = payload.data
        } catch {
          rawYamlData.value = payload.data
        }
      } else {
        rawManifest.value = payload.data as Record<string, unknown>
        rawYamlData.value = JSON.stringify(payload.data)
      }
      if (rawManifest.value && options.onLoaded) {
        options.onLoaded(rawManifest.value)
      }
    }
    isYamlLoading.value = false
  }

  const handleErrorOccurred = (payload: { message?: string }) => {
    if (!isYamlLoading.value) return
    const msg = payload?.message || ''
    const currentName = toValue(options.name)
    const currentKind = toValue(options.kind)
    const reqId = activeRequestId.value

    const isRelated =
      !msg ||
      !currentName ||
      (currentName && msg.includes(currentName)) ||
      (currentKind && msg.includes(currentKind)) ||
      (reqId && msg.includes(reqId))

    if (isRelated) {
      isYamlLoading.value = false
      yamlError.value = msg || 'Failed to fetch resource manifest'
    }
  }

  const handleBatchUpdated = (payload: {
    kind?: string
    updates?: Array<{ action: string; data?: { name?: string; namespace?: string } }>
  }) => {
    const isVis = options.visible !== undefined ? toValue(options.visible) : true
    if (!isVis) return

    const currentKind = toValue(options.kind)
    const currentName = toValue(options.name)
    const currentNs = toValue(options.namespace) || ''
    if (!currentName) return

    if (payload.kind && payload.kind.toLowerCase() !== currentKind.toLowerCase()) {
      return
    }

    if (Array.isArray(payload.updates)) {
      const matchingUpdate = payload.updates.find((u) => {
        const dataName = (u.data as { name?: string })?.name
        const dataNs = (u.data as { namespace?: string })?.namespace
        const nameMatch = dataName === currentName
        const nsMatch = !currentNs || !dataNs || dataNs === currentNs
        return nameMatch && nsMatch
      })

      if (!matchingUpdate) {
        return
      }

      if (matchingUpdate.action === 'DELETED') {
        return
      }
    }

    void fetchRawYaml()
  }

  const displayedYaml = computed(() => {
    if (rawManifest.value) {
      return yaml.stringify(rawManifest.value)
    }
    if (rawYamlData.value) {
      return rawYamlData.value
    }
    const fallback = toValue(options.fallbackResource)
    if (fallback) {
      return yaml.stringify(fallback)
    }
    return ''
  })

  const copyYaml = async () => {
    if (!displayedYaml.value) return
    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard) {
        await navigator.clipboard.writeText(displayedYaml.value)
      }
      copied.value = true
      if (copyTimeout) clearTimeout(copyTimeout)
      copyTimeout = setTimeout(() => {
        copied.value = false
      }, 2000)
    } catch (e) {
      console.error('Failed to copy YAML:', e)
    }
  }

  const initListeners = () => {
    events.on(OrbitEvents.ResourceRawData, handleRawData)
    events.on(OrbitEvents.ResourceBatchUpdated, handleBatchUpdated)
    events.on(OrbitEvents.ErrorOccurred, handleErrorOccurred)
  }

  const cleanup = () => {
    events.off(OrbitEvents.ResourceRawData, handleRawData)
    events.off(OrbitEvents.ResourceBatchUpdated, handleBatchUpdated)
    events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
    if (copyTimeout) {
      clearTimeout(copyTimeout)
      copyTimeout = null
    }
  }

  if (getCurrentInstance()) {
    onMounted(initListeners)
    onUnmounted(cleanup)
  } else {
    initListeners()
  }

  watch(
    () =>
      [
        options.visible !== undefined ? toValue(options.visible) : true,
        toValue(options.kind),
        toValue(options.name),
        toValue(options.namespace)
      ] as const,
    ([newVisible, , newName, newNs], prev) => {
      const oldVisible = prev?.[0]
      const oldName = prev?.[2]
      const oldNs = prev?.[3]
      if (newVisible && newName) {
        if (!oldVisible || newName !== oldName || newNs !== oldNs) {
          rawYamlData.value = null
          rawManifest.value = null
          void fetchRawYaml()
        }
      }
    },
    { immediate: true }
  )

  return {
    rawManifest,
    rawYamlData,
    displayedYaml,
    isYamlLoading,
    yamlError,
    copied,
    activeRequestId,
    fetchRawYaml,
    copyYaml,
    cleanup
  }
}
