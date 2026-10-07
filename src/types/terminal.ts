export type TerminalType = 'pod' | 'local'

export interface TerminalSession {
  id: string
  title: string
  type: TerminalType
  namespace?: string
  pod?: string
  container?: string
}

export interface OpenPodTerminalRequest {
  namespace: string
  pod: string
  container?: string
}

export interface OpenLocalTerminalRequest {
  shell?: string
}

export interface TerminalResponse {
  sessionId: string
}

export interface TerminalDataRequest {
  sessionId: string
  data: string
}

export interface TerminalResizeRequest {
  sessionId: string
  cols: number
  rows: number
}

export interface CloseTerminalRequest {
  sessionId: string
}

export interface TerminalDataEvent {
  sessionId: string
  data: string
}

export interface TerminalClosedEvent {
  sessionId: string
  exitCode?: number
}
