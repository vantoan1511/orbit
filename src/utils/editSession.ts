/**
 * Utility functions for edit resource session management and event correlation.
 */
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
