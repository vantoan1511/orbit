import EditorWorker from 'monaco-editor/editor/editor.worker?worker'

let configured = false

export function ensureMonacoEnvironment(): void {
  if (configured) return
  const globalScope = globalThis as typeof globalThis & {
    MonacoEnvironment?: {
      getWorker: (_workerId: string, _label: string) => Worker
    }
  }
  if (!globalScope.MonacoEnvironment) {
    globalScope.MonacoEnvironment = {
      getWorker: () => new EditorWorker()
    }
  }
  configured = true
}
