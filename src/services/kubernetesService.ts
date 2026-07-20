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
    await coreEngine.dispatch('getConfigMaps', { namespace })
  },

  /**
   * Request list of events, optionally filtered by namespace
   */
  async getEvents(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getEvents', { namespace })
  },

  /**
   * Request list of secrets, optionally filtered by namespace
   */
  async getSecrets(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getSecrets', { namespace })
  },

  /**
   * Request list of persistent volumes
   */
  async getPersistentVolumes(): Promise<void> {
    await coreEngine.dispatch('getPersistentVolumes')
  },

  /**
   * Request list of persistent volume claims, optionally filtered by namespace
   */
  async getPersistentVolumeClaims(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getPersistentVolumeClaims', { namespace })
  },

  /**
   * Request list of storage classes
   */
  async getStorageClasses(): Promise<void> {
    await coreEngine.dispatch('getStorageClasses')
  },

  /**
   * Request list of nodes
   */
  async getNodes(): Promise<void> {
    await coreEngine.dispatch('getNodes')
  },

  /**
   * Request list of policies, optionally filtered by namespace
   */
  async getPolicies(namespace?: string): Promise<void> {
    await coreEngine.dispatch('getPolicies', { namespace })
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
  },

  /**
   * Start streaming logs for a pod or workload container
   */
  async streamLogs(params: {
    namespace: string
    workload: string
    kind: string
    container?: string
    pod?: string
    tailLines?: number
  }): Promise<void> {
    await coreEngine.dispatch('streamLogs', params)
  },

  /**
   * Stop streaming logs
   */
  async stopLogs(): Promise<void> {
    await coreEngine.dispatch('stopLogs')
  },

  async scaleResource(params: {
    namespace: string
    kind: string
    name: string
    replicas: number
  }): Promise<void> {
    await coreEngine.dispatch('scaleResource', params)
  },

  async redeployResource(params: {
    namespace: string
    kind: string
    name: string
  }): Promise<void> {
    await coreEngine.dispatch('redeployResource', params)
  },

  async restartPod(params: {
    namespace: string
    name: string
  }): Promise<void> {
    await coreEngine.dispatch('restartPod', params)
  }
}
