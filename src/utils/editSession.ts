/**
 * Utility functions for edit resource session management and event correlation.
 */

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
