import assert from 'node:assert/strict'
import test from 'node:test'
import {
  isAddClusterModalVisible,
  openAddClusterModal,
  closeAddClusterModal,
  useCluster
} from '../useCluster.ts'

test('add cluster modal visibility controls toggle state', () => {
  closeAddClusterModal()
  assert.equal(isAddClusterModalVisible.value, false)

  openAddClusterModal()
  assert.equal(isAddClusterModalVisible.value, true)

  closeAddClusterModal()
  assert.equal(isAddClusterModalVisible.value, false)
})

test('handleAddCluster opens the add cluster modal', () => {
  closeAddClusterModal()
  const { handleAddCluster } = useCluster()
  handleAddCluster()
  assert.equal(isAddClusterModalVisible.value, true)
  closeAddClusterModal()
})
