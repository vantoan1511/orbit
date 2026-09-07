import assert from 'node:assert/strict'
import test from 'node:test'
import { formatCpuCores, formatDecimal, formatMemoryMiB } from '../metrics.ts'

test('formatDecimal formats up to maxDecimals without trailing zeros', () => {
  assert.equal(formatDecimal(300.254, 2), '300.25')
  assert.equal(formatDecimal(300.2, 2), '300.2')
  assert.equal(formatDecimal(300.0, 2), '300')
  assert.equal(formatDecimal(0, 2), '0')
  assert.equal(formatDecimal(2.25, 2), '2.25')
  assert.equal(formatDecimal(12.0, 2), '12')
  assert.equal(formatDecimal(-0, 2), '0')
  assert.equal(formatDecimal(-0.0001, 2), '0')
  assert.equal(formatDecimal(NaN, 2), '0')
})

test('formatMemoryMiB formats with Mi suffix', () => {
  assert.equal(formatMemoryMiB(300.25), '300.25Mi')
  assert.equal(formatMemoryMiB(300.0), '300Mi')
  assert.equal(formatMemoryMiB(45.5), '45.5Mi')
})

test('formatCpuCores formats cores or millicores', () => {
  assert.equal(formatCpuCores(2.25), '2.25 cores')
  assert.equal(formatCpuCores(1.0), '1 cores')
  assert.equal(formatCpuCores(0.25), '250m')
  assert.equal(formatCpuCores(0.05), '50m')
  assert.equal(formatCpuCores(0), '0m')
})
