import { OrbitEvents, type OrbitEventMap } from '@/types/events'
import { coreEngine, events } from './nativeService'

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
  applyUpdate(url: string) {
    coreEngine.dispatch('applyUpdate', { url })
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
