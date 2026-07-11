import { coreEngine } from './nativeService'

export const kubernetesService = {
  /**
   * Request list of namespaces from active cluster context
   */
  async getNamespaces(): Promise<void> {
    await coreEngine.dispatch('getNamespaces')
  },

  /**
   * Request list of pods, optionally filtered by namespace
   */
  async getPods(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getPods', { namespace })
  },

  /**
   * Request list of clusters
   */
  async getClusters(): Promise<void> {
    await coreEngine.dispatch('getClusters')
  },

  /**
   * Switch active context/cluster
   */
  async switchCluster(clusterId: string): Promise<void> {
    await coreEngine.dispatch('switchCluster', { clusterId })
  },

  /**
   * Add a new cluster kubeconfig file path
   */
  async addCluster(filePath: string): Promise<void> {
    await coreEngine.dispatch('addCluster', { filePath })
  }
}
