import type { Configuration, ConfigurationMap } from '@/types/settings'
import { coreEngine } from './nativeService'

export const appSettingsService = {
  /**
   * Request application settings from backend
   */
  async getAppSettings(): Promise<void> {
    await coreEngine.dispatch('getAppSettings')
  },

  /**
   * Update application settings in backend using unified Configuration array or key-value map
   */
  async updateAppSettings(settings: Configuration[] | ConfigurationMap): Promise<void> {
    await coreEngine.dispatch('updateAppSettings', settings)
  }
}
