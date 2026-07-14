import { OrbitEvents, type OrbitEventMap } from '@/types/events'
import { coreEngine, events } from './nativeService'

class UpdaterService {
  /**
   * Triggers a check for updates.
   */
  checkForUpdates(currentResources: string) {
    coreEngine.dispatch('checkForUpdates', { currentResources })
  }

  /**
   * Start downloading and applying a resource update (silent update).
   */
  applyResourceUpdate(url: string) {
    coreEngine.dispatch('applyResourceUpdate', { url })
  }

  /**
   * Start downloading and applying an engine update (requires restart).
   */
  triggerEngineUpdate(url: string) {
    coreEngine.dispatch('triggerEngineUpdate', { url })
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
