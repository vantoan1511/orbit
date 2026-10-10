import { OrbitEvents, type OrbitEventMap } from '../types/events.ts'
import { coreEngine, events } from './nativeService.ts'

class UpdaterService {
  /**
   * Triggers a check for updates.
   */
  checkForUpdates() {
    coreEngine.dispatch('checkForUpdates', {})
  }

  /**
   * Start downloading and applying an update (requires restart).
   */
  applyUpdate(url: string, version: string) {
    coreEngine.dispatch('applyUpdate', { url, version })
  }

  onUpdateCheckFinished(handler: (data: OrbitEventMap['updateCheckFinished']) => void) {
    return events.on(OrbitEvents.UpdateCheckFinished, handler)
  }

  onUpdateDownloadProgress(handler: (data: OrbitEventMap['updateDownloadProgress']) => void) {
    return events.on(OrbitEvents.UpdateDownloadProgress, handler)
  }

  onUpdateReady(handler: (data: OrbitEventMap['updateReady']) => void) {
    return events.on(OrbitEvents.UpdateReady, handler)
  }
}

export const updaterService = new UpdaterService()
