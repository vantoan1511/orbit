import assert from 'node:assert/strict'
import test from 'node:test'
import { storage } from '../nativeService.ts'
import {
  createDefaultFilters,
  TABLE_FILTER_STORAGE_KEY,
  tableFilterService,
  validatePersistedFilters
} from '../tableFilterService.ts'

test('createDefaultFilters returns valid default schema', () => {
  const defaults = createDefaultFilters()
  assert.equal(defaults.version, 1)
  assert.equal(defaults.defaultRows, 25)
  assert.deepEqual(defaults.tables, {})
  assert.deepEqual(defaults.clusters, {})
})

test('validatePersistedFilters handles invalid inputs gracefully', () => {
  assert.deepEqual(validatePersistedFilters(null), createDefaultFilters())
  assert.deepEqual(validatePersistedFilters(undefined), createDefaultFilters())
  assert.deepEqual(validatePersistedFilters('string'), createDefaultFilters())
  assert.deepEqual(validatePersistedFilters(123), createDefaultFilters())
  assert.deepEqual(validatePersistedFilters([]), createDefaultFilters())
})

test('validatePersistedFilters validates defaultRows correctly', () => {
  // Valid row numbers: 25, 50, 100, 200
  const valid50 = validatePersistedFilters({ defaultRows: 50 })
  assert.equal(valid50.defaultRows, 50)

  const valid100 = validatePersistedFilters({ defaultRows: 100 })
  assert.equal(valid100.defaultRows, 100)

  // Invalid row numbers fall back to 25
  const invalid15 = validatePersistedFilters({ defaultRows: 15 })
  assert.equal(invalid15.defaultRows, 25)

  const invalidString = validatePersistedFilters({ defaultRows: '50' })
  assert.equal(invalidString.defaultRows, 25)
})

test('validatePersistedFilters protects tables and clusters from non-object values', () => {
  const result = validatePersistedFilters({
    version: 2,
    defaultRows: 50,
    tables: ['not', 'an', 'object'],
    clusters: null
  })

  assert.equal(result.version, 2)
  assert.equal(result.defaultRows, 50)
  assert.deepEqual(result.tables, {})
  assert.deepEqual(result.clusters, {})
})

test('validatePersistedFilters preserves valid tables and clusters', () => {
  const sample = {
    version: 1,
    defaultRows: 50,
    tables: {
      pod: { rows: 50, columns: [{ field: 'name', header: 'Name', visible: true }] }
    },
    clusters: {
      'test-cluster': {
        pod: { searchQuery: 'nginx', selectedNamespace: ['default'] }
      }
    }
  }

  const validated = validatePersistedFilters(sample)
  assert.deepEqual(validated, sample)
})

test('validatePersistedFilters strips invalid nested entries from tables and clusters', () => {
  const sample = {
    version: 1,
    defaultRows: 25,
    tables: {
      validTable: { rows: 50 },
      corruptNull: null,
      corruptString: 'bad',
      corruptArray: [1, 2, 3]
    },
    clusters: {
      validCluster: { pod: { searchQuery: 'test' } },
      corruptCluster: null,
      corruptPrimitive: 42
    }
  }

  const validated = validatePersistedFilters(sample)
  assert.deepEqual(validated.tables, { validTable: { rows: 50 } })
  assert.deepEqual(validated.clusters, { validCluster: { pod: { searchQuery: 'test' } } })
})

test('tableFilterService.loadFilters loads from native storage when present', async () => {
  const originalGetData = storage.getData
  const sample = {
    version: 1,
    defaultRows: 100,
    tables: {},
    clusters: {}
  }

  storage.getData = async (key: string) => {
    assert.equal(key, TABLE_FILTER_STORAGE_KEY)
    return JSON.stringify(sample)
  }

  try {
    const loaded = await tableFilterService.loadFilters()
    assert.equal(loaded.defaultRows, 100)
  } finally {
    storage.getData = originalGetData
  }
})

test('tableFilterService.loadFilters performs legacy migration from localStorage', async () => {
  const originalGetData = storage.getData
  const originalSetData = storage.setData

  // Mock native storage rejecting (key not yet in native storage)
  storage.getData = async () => {
    throw new Error('NE_ST_NOKEY')
  }

  let savedToNativeKey = ''
  let savedToNativeData: string | null = null
  storage.setData = async (key: string, data?: string | null) => {
    savedToNativeKey = key
    savedToNativeData = data ?? null
  }

  // Setup mock localStorage
  const legacyData = JSON.stringify({
    version: 1,
    defaultRows: 50,
    tables: { deployment: { rows: 50 } },
    clusters: {}
  })

  let removedItemKey = ''
  const mockLocalStorage = {
    getItem(key: string) {
      return key === TABLE_FILTER_STORAGE_KEY ? legacyData : null
    },
    removeItem(key: string) {
      removedItemKey = key
    },
    setItem() {}
  }

  const originalLocalStorage = globalThis.localStorage
  Object.defineProperty(globalThis, 'localStorage', {
    value: mockLocalStorage,
    configurable: true,
    writable: true
  })

  try {
    const loaded = await tableFilterService.loadFilters()
    assert.equal(loaded.defaultRows, 50)
    assert.equal(savedToNativeKey, TABLE_FILTER_STORAGE_KEY)
    assert.equal(savedToNativeData, legacyData)
    assert.equal(removedItemKey, TABLE_FILTER_STORAGE_KEY)
  } finally {
    storage.getData = originalGetData
    storage.setData = originalSetData
    Object.defineProperty(globalThis, 'localStorage', {
      value: originalLocalStorage,
      configurable: true,
      writable: true
    })
  }
})

test('tableFilterService.saveFilters writes to native storage', async () => {
  const originalSetData = storage.setData
  let savedKey = ''
  let savedData: string | null = null

  storage.setData = async (key: string, data?: string | null) => {
    savedKey = key
    savedData = data ?? null
  }

  const sample = createDefaultFilters()
  sample.defaultRows = 200

  try {
    await tableFilterService.saveFilters(sample)
    assert.equal(savedKey, TABLE_FILTER_STORAGE_KEY)
    assert.ok(savedData)
    const parsed = JSON.parse(savedData)
    assert.equal(parsed.defaultRows, 200)
  } finally {
    storage.setData = originalSetData
  }
})

test('tableFilterService.clearFilters removes data from native storage', async () => {
  const originalRemoveData = storage.removeData
  let removedKey = ''

  storage.removeData = async (key: string) => {
    removedKey = key
  }

  try {
    await tableFilterService.clearFilters()
    assert.equal(removedKey, TABLE_FILTER_STORAGE_KEY)
  } finally {
    storage.removeData = originalRemoveData
  }
})

test('tableFilterService.loadFilters falls back to defaults when native storage times out', async () => {
  const originalGetData = storage.getData

  // Simulate a hanging promise that does not resolve
  storage.getData = () => new Promise<string>(() => {})

  try {
    const loaded = await tableFilterService.loadFilters()
    assert.deepEqual(loaded, createDefaultFilters())
  } finally {
    storage.getData = originalGetData
  }
})
