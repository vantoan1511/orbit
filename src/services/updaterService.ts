import { coreEngine, events } from './nativeService'
import { OrbitEvents } from '@/types/events'

class UpdaterService {
  /**
   * Triggers a check for updates.
   */
  checkForUpdates() {
    coreEngine.dispatch('checkForUpdates')
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

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onUpdateCheckFinished(handler: (data: any) => void) {
    return events.on(OrbitEvents.UpdateCheckFinished, handler)
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onUpdateDownloadProgress(handler: (data: any) => void) {
    return events.on(OrbitEvents.UpdateDownloadProgress, handler)
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onUpdateReady(handler: (data: any) => void) {
    return events.on(OrbitEvents.UpdateReady, handler)
  }
}

export const updaterService = new UpdaterService()
