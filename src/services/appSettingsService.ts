import type { OrbitConfig } from '@/types/settings'
import { coreEngine } from './nativeService'

export const appSettingsService = {
  /**
   * Request application settings from backend
   */
  async getAppSettings(): Promise<void> {
    await coreEngine.dispatch('getAppSettings')
  },

  /**
   * Update application settings in backend
   */
  async updateAppSettings(settings: OrbitConfig): Promise<void> {
    await coreEngine.dispatch('updateAppSettings', settings)
  }
}
