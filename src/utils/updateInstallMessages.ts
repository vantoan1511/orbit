import type { UpdateInstallFailureCode } from '../types/events.ts'

export const UPDATE_INSTALL_MESSAGES: Record<UpdateInstallFailureCode, string> = {
  elevationDenied: 'Update cancelled — administrator permission was not granted',
  launchFailed: 'Failed to launch installer'
}

export function formatUpdateInstallErrorMessage(payload: {
  code?: UpdateInstallFailureCode
  message?: string
}): string {
  if (payload.code && payload.code in UPDATE_INSTALL_MESSAGES) {
    if (payload.code === 'launchFailed' && payload.message) {
      return payload.message
    }
    return UPDATE_INSTALL_MESSAGES[payload.code]
  }
  return payload.message || 'Failed to apply update'
}
