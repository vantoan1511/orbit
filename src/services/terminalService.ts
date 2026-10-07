import { coreEngine } from './nativeService.ts'

export const terminalService = {
  /**
   * Request to open a local system shell terminal session.
   */
  async openLocalTerminal(sessionId: string, shell?: string): Promise<void> {
    await coreEngine.dispatch('openLocalTerminal', { sessionId, shell })
  },

  /**
   * Request to open a pod exec terminal session.
   */
  async openPodTerminal(
    sessionId: string,
    namespace: string,
    pod: string,
    container?: string
  ): Promise<void> {
    await coreEngine.dispatch('openPodTerminal', {
      sessionId,
      namespace,
      pod,
      container
    })
  },

  /**
   * Send stdin character data to the terminal backend.
   */
  async sendData(sessionId: string, data: string): Promise<void> {
    await coreEngine.dispatch('sendTerminalData', { sessionId, data })
  },

  /**
   * Request resizing the terminal PTY / container TTY.
   */
  async resize(sessionId: string, cols: number, rows: number): Promise<void> {
    await coreEngine.dispatch('resizeTerminal', { sessionId, cols, rows })
  },

  /**
   * Close a terminal session.
   */
  async close(sessionId: string): Promise<void> {
    await coreEngine.dispatch('closeTerminal', { sessionId })
  }
}
