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
   * Request list of deployments, optionally filtered by namespace
   */
  async getDeployments(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getDeployments', { namespace })
  },

  /**
   * Request list of statefulsets, optionally filtered by namespace
   */
  async getStatefulSets(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getStatefulSets', { namespace })
  },

  /**
   * Request list of daemonsets, optionally filtered by namespace
   */
  async getDaemonSets(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getDaemonSets', { namespace })
  },

  /**
   * Request list of replicasets, optionally filtered by namespace
   */
  async getReplicaSets(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getReplicaSets', { namespace })
  },

  /**
   * Request list of jobs, optionally filtered by namespace
   */
  async getJobs(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getJobs', { namespace })
  },

  /**
   * Request list of cronjobs, optionally filtered by namespace
   */
  async getCronJobs(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getCronJobs', { namespace })
  },

  /**
   * Request list of services, optionally filtered by namespace
   */
  async getServices(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getServices', { namespace })
  },

  /**
   * Request list of configmaps, optionally filtered by namespace
   */
  async getConfigMaps(namespace?: string): Promise<void> {
    try {
      const { useKubernetesStore } = await import('@/stores/kubernetesStore')
      const store = useKubernetesStore()
      store.setConfigMapsLoading(true)
      await coreEngine.dispatch('getConfigMaps', { namespace })
    } catch (error) {
      console.error('Failed to get ConfigMaps:', error)
      try {
        const { useKubernetesStore } = await import('@/stores/kubernetesStore')
        const store = useKubernetesStore()
        store.setConfigMapsLoading(false)
      } catch {}
      throw error
    }
  },

  /**
   * Request list of secrets, optionally filtered by namespace
   */
  async getSecrets(namespace?: string): Promise<void> {
    try {
      const { useKubernetesStore } = await import('@/stores/kubernetesStore')
      const store = useKubernetesStore()
      store.setSecretsLoading(true)
      await coreEngine.dispatch('getSecrets', { namespace })
    } catch (error) {
      console.error('Failed to get Secrets:', error)
      try {
        const { useKubernetesStore } = await import('@/stores/kubernetesStore')
        const store = useKubernetesStore()
        store.setSecretsLoading(false)
      } catch {}
      throw error
    }
  },

  /**
   * Request list of nodes
   */
  async getNodes(): Promise<void> {
    await coreEngine.dispatch('getNodes')
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
