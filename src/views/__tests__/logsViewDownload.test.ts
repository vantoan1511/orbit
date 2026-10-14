import test from 'node:test'
import assert from 'node:assert/strict'
import { handleDownloadLogsNotification } from '../../utils/logsViewHelpers.ts'

test('handleDownloadLogsNotification displays success toast when logs are saved', () => {
  const addedToasts: Array<{ severity: string; summary: string; detail: string; life?: number }> =
    []
  const mockToast = {
    add: (msg: { severity: string; summary: string; detail: string; life?: number }) => {
      addedToasts.push(msg)
    }
  }

  handleDownloadLogsNotification(
    { status: 'saved', filePath: 'C:\\Users\\Desktop\\my-workload-logs.txt' },
    mockToast
  )

  assert.equal(addedToasts.length, 1)
  assert.equal(addedToasts[0].severity, 'success')
  assert.equal(addedToasts[0].summary, 'Logs Saved')
  assert.ok(addedToasts[0].detail.includes('my-workload-logs.txt'))
  assert.equal(addedToasts[0].life, 3000)
})

test('handleDownloadLogsNotification displays error toast when save fails', () => {
  const addedToasts: Array<{ severity: string; summary: string; detail: string; life?: number }> =
    []
  const mockToast = {
    add: (msg: { severity: string; summary: string; detail: string; life?: number }) => {
      addedToasts.push(msg)
    }
  }

  handleDownloadLogsNotification({ status: 'error', error: 'Disk full error' }, mockToast)

  assert.equal(addedToasts.length, 1)
  assert.equal(addedToasts[0].severity, 'error')
  assert.equal(addedToasts[0].summary, 'Save Failed')
  assert.ok(addedToasts[0].detail.includes('Disk full error'))
  assert.equal(addedToasts[0].life, 4000)
})

test('handleDownloadLogsNotification displays no toast when user cancels', () => {
  const addedToasts: Array<{ severity: string; summary: string; detail: string; life?: number }> =
    []
  const mockToast = {
    add: (msg: { severity: string; summary: string; detail: string; life?: number }) => {
      addedToasts.push(msg)
    }
  }

  handleDownloadLogsNotification({ status: 'cancelled' }, mockToast)

  assert.equal(addedToasts.length, 0)
})
