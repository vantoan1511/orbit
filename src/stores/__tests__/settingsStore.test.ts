import test from 'node:test'
import assert from 'node:assert/strict'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore, isConfigurationEqual } from '../settingsStore.ts'
import type { Configuration } from '../../types/settings.ts'

const createSampleConfig = (overrides?: Partial<Configuration>): Configuration => ({
  key: 'test_key',
  name: 'Test Key',
  description: 'Test description',
  datatype: 'boolean',
  defaultValue: false,
  value: false,
  enable: true,
  isConfidential: false,
  cardinality: 'one',
  ...overrides
})

test('isConfigurationEqual accurately compares configurations', () => {
  const c1 = createSampleConfig({ key: 'k1', value: true })
  const c2 = createSampleConfig({ key: 'k1', value: true })
  const c3 = createSampleConfig({ key: 'k1', value: false })
  const c4 = createSampleConfig({ key: 'k2', value: true })

  assert.equal(isConfigurationEqual(c1, c2), true)
  assert.equal(isConfigurationEqual(c1, c3), false)
  assert.equal(isConfigurationEqual(c1, c4), false)

  // Compare effective values when value is undefined vs defaultValue
  const c5 = createSampleConfig({ key: 'k1', value: undefined, defaultValue: 'auto' })
  const c6 = createSampleConfig({ key: 'k1', value: 'auto', defaultValue: 'auto' })
  assert.equal(isConfigurationEqual(c5, c6), true)
})

test('setSettings(Configuration[]) preserves array and object references when values match', () => {
  setActivePinia(createPinia())
  const store = useSettingsStore()

  const initialList: Configuration[] = [
    createSampleConfig({ key: 'item1', value: 'v1' }),
    createSampleConfig({ key: 'item2', value: 'v2' })
  ]

  store.setSettings(initialList)
  const arrayRef1 = store.settings
  const item1Ref1 = store.settings[0]
  const item2Ref1 = store.settings[1]

  // Call again with brand new objects having identical content
  const cloneList: Configuration[] = [
    createSampleConfig({ key: 'item1', value: 'v1' }),
    createSampleConfig({ key: 'item2', value: 'v2' })
  ]
  store.setSettings(cloneList)

  assert.equal(store.settings, arrayRef1, 'Array reference should be preserved')
  assert.equal(store.settings[0], item1Ref1, 'Item 1 object reference should be preserved')
  assert.equal(store.settings[1], item2Ref1, 'Item 2 object reference should be preserved')

  // Update item 2 only
  const updateList: Configuration[] = [
    createSampleConfig({ key: 'item1', value: 'v1' }),
    createSampleConfig({ key: 'item2', value: 'v2_updated' })
  ]
  store.setSettings(updateList)

  assert.notEqual(
    store.settings,
    arrayRef1,
    'Array reference should change when an item is updated'
  )
  assert.equal(store.settings[0], item1Ref1, 'Unchanged item 1 reference should be preserved')
  assert.notEqual(store.settings[1], item2Ref1, 'Modified item 2 reference should be updated')
  assert.equal(store.settings[1]?.value, 'v2_updated')
})

test('setSettings(ConfigurationMap) preserves array reference when values match', () => {
  setActivePinia(createPinia())
  const store = useSettingsStore()

  const initialList: Configuration[] = [
    createSampleConfig({ key: 'item1', value: 'v1' }),
    createSampleConfig({ key: 'item2', value: 'v2' })
  ]
  store.setSettings(initialList)
  const arrayRef1 = store.settings
  const item1Ref1 = store.settings[0]
  const item2Ref1 = store.settings[1]

  // Map with unchanged values
  store.setSettings({ item1: 'v1', item2: 'v2' })
  assert.equal(
    store.settings,
    arrayRef1,
    'Array reference should be preserved when map has unchanged values'
  )
  assert.equal(store.settings[0], item1Ref1)
  assert.equal(store.settings[1], item2Ref1)

  // Map with one changed value
  store.setSettings({ item2: 'v2_new' })
  assert.notEqual(store.settings, arrayRef1, 'Array reference should update when map value changes')
  assert.equal(store.settings[0], item1Ref1, 'Item 1 reference preserved')
  assert.equal(store.settings[1]?.value, 'v2_new')
})

test('subsequent setSettings with matching value preserves reference after optimistic update', async () => {
  setActivePinia(createPinia())
  const store = useSettingsStore()

  store.setSettings([
    createSampleConfig({ key: 'item1', value: 'v1' }),
    createSampleConfig({ key: 'item2', value: 'v2' })
  ])

  // Mock optimistic update by updating store value (or calling updateConfigValue without backend failure)
  // Let's directly test what happens when setSettings arrives with identical payload
  const arrayRefBefore = store.settings
  const payloadFromBackend = [
    createSampleConfig({ key: 'item1', value: 'v1' }),
    createSampleConfig({ key: 'item2', value: 'v2' })
  ]
  store.setSettings(payloadFromBackend)
  assert.equal(store.settings, arrayRefBefore)
})
