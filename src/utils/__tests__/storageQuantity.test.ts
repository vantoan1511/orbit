import assert from 'node:assert/strict'
import test from 'node:test'
import {
  parseStorageBytes,
  parseCapacityToGiB,
  isValidStorageQuantity,
  formatStorageGiB,
  formatDonutTotalCapacity
} from '../storageQuantity.ts'

test('parseStorageBytes and parseCapacityToGiB with binary IEC units', () => {
  // 512Ki = 512 * 1024 bytes = 524,288 bytes
  assert.equal(parseStorageBytes('512Ki'), 512 * 1024)
  assert.equal(parseCapacityToGiB('512Ki'), (512 * 1024) / 1024 ** 3)

  // 500Mi = 500 * 1024^2 bytes
  assert.equal(parseStorageBytes('500Mi'), 500 * 1024 * 1024)
  assert.equal(parseCapacityToGiB('500Mi'), 500 / 1024)

  // 10Gi = 10 * 1024^3 bytes = 10 GiB
  assert.equal(parseStorageBytes('10Gi'), 10 * 1024 ** 3)
  assert.equal(parseCapacityToGiB('10Gi'), 10)

  // 1Ti = 1024 GiB
  assert.equal(parseStorageBytes('1Ti'), 1024 ** 4)
  assert.equal(parseCapacityToGiB('1Ti'), 1024)

  // 2Pi = 2 * 1024^2 GiB
  assert.equal(parseStorageBytes('2Pi'), 2 * 1024 ** 5)
  assert.equal(parseCapacityToGiB('2Pi'), 2 * 1024 * 1024)

  // Explicit *iB variants
  assert.equal(parseCapacityToGiB('500MiB'), 500 / 1024)
  assert.equal(parseCapacityToGiB('10GiB'), 10)
  assert.equal(parseCapacityToGiB('1TiB'), 1024)
  assert.equal(parseCapacityToGiB('1KiB'), 1 / (1024 * 1024))
  assert.equal(parseCapacityToGiB('1PiB'), 1024 * 1024)
})

test('parseStorageBytes and parseCapacityToGiB with decimal SI units', () => {
  // 500M = 500 * 10^6 bytes
  assert.equal(parseStorageBytes('500M'), 500 * 1000 * 1000)
  assert.ok(Math.abs(parseCapacityToGiB('500M') - 500000000 / 1024 ** 3) < 1e-6)

  // 100G = 100 * 10^9 bytes
  assert.equal(parseStorageBytes('100G'), 100 * 1000 ** 3)
  assert.ok(Math.abs(parseCapacityToGiB('100G') - 100000000000 / 1024 ** 3) < 1e-6)

  // 1T = 10^12 bytes
  assert.equal(parseStorageBytes('1T'), 1000 ** 4)
  assert.ok(Math.abs(parseCapacityToGiB('1T') - 1000000000000 / 1024 ** 3) < 1e-6)

  // 1000k and 500K
  assert.equal(parseStorageBytes('1000k'), 1000 * 1000)
  assert.equal(parseStorageBytes('500K'), 500 * 1000)

  // 100GB, 500MB
  assert.equal(parseStorageBytes('100GB'), 100 * 1000 ** 3)
  assert.equal(parseStorageBytes('500MB'), 500 * 1000 ** 2)

  // 500m (millibytes = 500 * 10^-3 bytes = 0.5 bytes)
  assert.equal(parseStorageBytes('500m'), 0.5)
  assert.ok(parseCapacityToGiB('500m') > 0)
})

test('parseStorageBytes and parseCapacityToGiB with raw numbers & scientific notation', () => {
  // 1073741824 = 1 GiB exactly
  assert.equal(parseStorageBytes('1073741824'), 1073741824)
  assert.equal(parseCapacityToGiB('1073741824'), 1)

  // scientific notation
  assert.equal(parseStorageBytes('1e9'), 1e9)
  assert.equal(parseStorageBytes('0'), 0)
  assert.equal(parseCapacityToGiB('0'), 0)
})

test('parseStorageBytes and parseCapacityToGiB with invalid or edge inputs return 0', () => {
  assert.equal(parseStorageBytes(null as unknown as string), 0)
  assert.equal(parseStorageBytes(undefined as unknown as string), 0)
  assert.equal(parseStorageBytes(''), 0)
  assert.equal(parseStorageBytes('   '), 0)
  assert.equal(parseStorageBytes('-'), 0)
  assert.equal(parseStorageBytes('-10Gi'), 0)
  assert.equal(parseStorageBytes('abc'), 0)
  assert.equal(parseStorageBytes('10Xi'), 0)

  assert.equal(parseCapacityToGiB(null as unknown as string), 0)
  assert.equal(parseCapacityToGiB(undefined as unknown as string), 0)
  assert.equal(parseCapacityToGiB(''), 0)
  assert.equal(parseCapacityToGiB('   '), 0)
  assert.equal(parseCapacityToGiB('-'), 0)
  assert.equal(parseCapacityToGiB('-10Gi'), 0)
  assert.equal(parseCapacityToGiB('abc'), 0)
  assert.equal(parseCapacityToGiB('10Xi'), 0)
})

test('isValidStorageQuantity validates positive quantities', () => {
  assert.equal(isValidStorageQuantity('10Gi'), true)
  assert.equal(isValidStorageQuantity('500Mi'), true)
  assert.equal(isValidStorageQuantity('100G'), true)
  assert.equal(isValidStorageQuantity('1.5Ti'), true)
  assert.equal(isValidStorageQuantity('1073741824'), true)

  assert.equal(isValidStorageQuantity(''), false)
  assert.equal(isValidStorageQuantity('   '), false)
  assert.equal(isValidStorageQuantity('-'), false)
  assert.equal(isValidStorageQuantity('0'), false)
  assert.equal(isValidStorageQuantity('0Gi'), false)
  assert.equal(isValidStorageQuantity('-5Gi'), false)
  assert.equal(isValidStorageQuantity('10Xi'), false)
  assert.equal(isValidStorageQuantity('abc'), false)
  assert.equal(isValidStorageQuantity(null as unknown as string), false)
  assert.equal(isValidStorageQuantity(undefined as unknown as string), false)
})

test('formatStorageGiB formats across MiB, GiB, and TiB ranges', () => {
  assert.equal(formatStorageGiB(0), '0.0 GiB')
  assert.equal(formatStorageGiB(0.5), '512 MiB')
  assert.equal(formatStorageGiB(10), '10.0 GiB')
  assert.equal(formatStorageGiB(1024), '1.0 TiB')
  assert.equal(formatStorageGiB(2048), '2.0 TiB')
})

test('formatDonutTotalCapacity returns formatted value and unit', () => {
  assert.deepEqual(formatDonutTotalCapacity(0), { value: '0', unit: 'Total GiB' })
  assert.deepEqual(formatDonutTotalCapacity(125.4), { value: '125.4', unit: 'Total GiB' })
  assert.deepEqual(formatDonutTotalCapacity(2048), { value: '2', unit: 'Total TiB' })
})
