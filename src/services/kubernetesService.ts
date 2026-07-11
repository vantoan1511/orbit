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
