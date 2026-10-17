export type ShortcutActionId =
  | 'quickOpen'
  | 'closeTab'
  | 'nextTab'
  | 'prevTab'
  | 'toggleSidebar'
  | 'toggleTerminal'
  | 'openSettings'

export type ShortcutCategory = 'Navigation' | 'Tabs' | 'Workspace' | 'Tools'

export interface ShortcutDefinition {
  id: ShortcutActionId
  name: string
  description: string
  category: ShortcutCategory
  defaultKey: string // Win / Linux (e.g. 'Ctrl+P')
  defaultKeyMac: string // macOS (e.g. '⌘P')
}

export interface KeyCombo {
  key: string // Primary normalized key: 'P', 'W', 'Tab', ',', '`', etc.
  ctrlKey: boolean
  metaKey: boolean
  shiftKey: boolean
  altKey: boolean
}

export type ShortcutBindings = Record<ShortcutActionId, string>
