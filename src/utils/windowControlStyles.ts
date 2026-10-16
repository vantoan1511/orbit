export type WindowControlAction = 'minimize' | 'maximize' | 'close'

export function getWindowControlButtonClass(action: WindowControlAction): string {
  const base =
    'h-8 w-11 inline-flex items-center justify-center rounded-none border-none p-0 bg-transparent text-muted-color transition-colors cursor-pointer select-none shrink-0'

  if (action === 'close') {
    return `${base} hover:bg-red-500/80 hover:text-white dark:hover:bg-red-600/90 active:bg-red-600`
  }

  return `${base} hover:bg-(--bg-hover) hover:text-primary active:bg-(--bg-active)`
}

export function getMaximizeTooltip(isMaximized: boolean): string {
  return isMaximized ? 'Restore' : 'Maximize'
}

export function getMaximizeAriaLabel(isMaximized: boolean): string {
  return isMaximized ? 'Restore window' : 'Maximize window'
}
