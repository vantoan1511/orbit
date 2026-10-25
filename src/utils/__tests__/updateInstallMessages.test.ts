import assert from 'node:assert/strict'
import test from 'node:test'
import {
  UPDATE_INSTALL_MESSAGES,
  formatUpdateInstallErrorMessage
} from '../updateInstallMessages.ts'

test('both UpdateInstallFailureCode values return non-empty distinct strings', () => {
  const elevationMsg = UPDATE_INSTALL_MESSAGES.elevationDenied
  const launchMsg = UPDATE_INSTALL_MESSAGES.launchFailed

  assert.ok(elevationMsg && elevationMsg.length > 0)
  assert.ok(launchMsg && launchMsg.length > 0)
  assert.notEqual(elevationMsg, launchMsg)
})

test('formatUpdateInstallErrorMessage returns elevationDenied message', () => {
  const msg = formatUpdateInstallErrorMessage({
    code: 'elevationDenied',
    message: 'ignored'
  })
  assert.equal(msg, UPDATE_INSTALL_MESSAGES.elevationDenied)
})

test('formatUpdateInstallErrorMessage falls back to message when code is unknown', () => {
  const msg = formatUpdateInstallErrorMessage({
    // @ts-expect-error testing unknown code fallback
    code: 'unknownCode',
    message: 'Custom fallback message'
  })
  assert.equal(msg, 'Custom fallback message')
})

test('formatUpdateInstallErrorMessage uses payload message for launchFailed if provided', () => {
  const msg = formatUpdateInstallErrorMessage({
    code: 'launchFailed',
    message: 'Failed to launch installer: Access denied'
  })
  assert.equal(msg, 'Failed to launch installer: Access denied')
})
