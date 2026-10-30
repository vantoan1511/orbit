/**
 * Utility functions for edit resource session management and event correlation.
 */
import * as yaml from 'yaml'
import type { KubernetesResource } from '../types/kubernetes.ts'

let counter = 0

export function buildEditRequestId(
  kind: string,
  namespace: string,
  name: string,
  nonce?: string | number
): string {
  const n = nonce !== undefined ? nonce : `${Date.now()}-${++counter}`
  return `edit:${kind}:${namespace}:${name}:${n}`
}

export interface EditResponsePayload {
  kind: string
  name: string
  namespace?: string
  requestId?: string
}

export interface EditSessionContext {
  kind: string
  namespace: string
  name: string
  requestId?: string
}

export function matchesEditResponse(
  payload: EditResponsePayload,
  ctx: EditSessionContext
): boolean {
  if (ctx.requestId && payload.requestId) {
    return ctx.requestId === payload.requestId
  }

  if (payload.requestId && !ctx.requestId) {
    return false
  }

  if (payload.kind !== ctx.kind || payload.name !== ctx.name) {
    return false
  }

  if (payload.namespace !== undefined && payload.namespace !== ctx.namespace) {
    return false
  }

  return true
}

export function shouldHandleCommandResult(isSaving: boolean): boolean {
  return isSaving
}

export interface EditApplyResultPayload {
  kind: string
  name: string
  namespace?: string
  requestId?: string
  success: boolean
  data?: Record<string, unknown>
  error?: string
}

export interface EditSaveContext {
  isSaving: boolean
  kind: string
  namespace: string
  name: string
  requestId?: string
}

export function matchesSaveResult(payload: EditApplyResultPayload, ctx: EditSaveContext): boolean {
  if (!ctx.isSaving) {
    return false
  }

  if (ctx.requestId) {
    if (!payload.requestId || payload.requestId !== ctx.requestId) {
      return false
    }
  }

  return matchesEditResponse(payload, ctx)
}

export interface ResourceMetadataFormValues {
  name: string
  namespace: string
  labels: { key: string; value: string }[]
  annotations: { key: string; value: string }[]
}

export function extractResourceMetadataFormValues(
  resource: KubernetesResource | Record<string, unknown> | null | undefined
): ResourceMetadataFormValues {
  const metadata =
    (resource as { metadata?: Record<string, unknown> } | null | undefined)?.metadata || {}
  const name = typeof metadata.name === 'string' ? metadata.name : ''
  const namespace = typeof metadata.namespace === 'string' ? metadata.namespace : ''
  const lbls =
    metadata.labels && typeof metadata.labels === 'object'
      ? (metadata.labels as Record<string, unknown>)
      : {}
  const labels = Object.entries(lbls).map(([key, value]) => ({
    key,
    value: String(value)
  }))
  const anns =
    metadata.annotations && typeof metadata.annotations === 'object'
      ? (metadata.annotations as Record<string, unknown>)
      : {}
  const annotations = Object.entries(anns).map(([key, value]) => ({
    key,
    value: String(value)
  }))

  return { name, namespace, labels, annotations }
}

export function applyMetadataFormValuesToResource(
  resource: KubernetesResource | Record<string, unknown> | null | undefined,
  formValues: ResourceMetadataFormValues
): KubernetesResource {
  const baseResource = (resource && typeof resource === 'object' ? resource : {}) as Record<
    string,
    unknown
  >
  const existingMetadata = (
    baseResource.metadata && typeof baseResource.metadata === 'object' ? baseResource.metadata : {}
  ) as Record<string, unknown>

  const labels: Record<string, string> = {}
  for (const item of formValues.labels || []) {
    const trimmedKey = typeof item?.key === 'string' ? item.key.trim() : ''
    if (trimmedKey) {
      labels[trimmedKey] = typeof item.value === 'string' ? item.value : String(item.value ?? '')
    }
  }

  const annotations: Record<string, string> = {}
  for (const item of formValues.annotations || []) {
    const trimmedKey = typeof item?.key === 'string' ? item.key.trim() : ''
    if (trimmedKey) {
      annotations[trimmedKey] =
        typeof item.value === 'string' ? item.value : String(item.value ?? '')
    }
  }

  const updatedMetadata: Record<string, unknown> = {
    ...existingMetadata,
    name:
      formValues.name || (typeof existingMetadata.name === 'string' ? existingMetadata.name : ''),
    labels,
    annotations
  }

  const resolvedNs =
    formValues.namespace ||
    (typeof existingMetadata.namespace === 'string' ? existingMetadata.namespace : '')
  if (resolvedNs && resolvedNs.length > 0) {
    updatedMetadata.namespace = resolvedNs
  } else {
    delete updatedMetadata.namespace
  }

  return {
    ...baseResource,
    metadata: updatedMetadata
  } as KubernetesResource
}

export interface EditModeTransitionInput {
  targetYamlMode: boolean
  rawData: KubernetesResource | null
  yamlContent: string
  hasCustomForm: boolean
}

export type EditModeTransitionResult =
  | {
      ok: true
      isYamlMode: boolean
      rawData: KubernetesResource | null
      yamlContent: string
      formValues?: ResourceMetadataFormValues
    }
  | {
      ok: false
      isYamlMode: true
      errorMessage: string
    }

export function transitionEditMode(input: EditModeTransitionInput): EditModeTransitionResult {
  if (input.targetYamlMode) {
    const nextYaml = input.rawData ? yaml.stringify(input.rawData) : input.yamlContent
    return {
      ok: true,
      isYamlMode: true,
      rawData: input.rawData,
      yamlContent: nextYaml
    }
  }

  try {
    const parsed = yaml.parse(input.yamlContent)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return {
        ok: false,
        isYamlMode: true,
        errorMessage: 'Parsed YAML must be a valid Kubernetes object'
      }
    }

    const kubernetesResource = parsed as KubernetesResource
    return {
      ok: true,
      isYamlMode: false,
      rawData: kubernetesResource,
      yamlContent: input.yamlContent,
      formValues: input.hasCustomForm
        ? undefined
        : extractResourceMetadataFormValues(kubernetesResource)
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    return {
      ok: false,
      isYamlMode: true,
      errorMessage: message
    }
  }
}
