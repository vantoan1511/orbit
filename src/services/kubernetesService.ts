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
  }
}
