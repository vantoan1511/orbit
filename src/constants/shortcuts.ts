import type { ShortcutBindings, ShortcutDefinition } from '../types/shortcuts.ts'

export const DEFAULT_SHORTCUT_DEFINITIONS: readonly ShortcutDefinition[] = [
  {
    id: 'quickOpen',
    name: 'Quick Open / Search',
    description: 'Open the search dialog to quickly find resources, logs, and navigation items.',
    category: 'Navigation',
    defaultKey: 'Ctrl+P',
    defaultKeyMac: '⌘P'
  },
  {
    id: 'closeTab',
    name: 'Close Active Tab',
    description: 'Close the currently selected workspace tab.',
    category: 'Tabs',
    defaultKey: 'Ctrl+W',
    defaultKeyMac: '⌘W'
  },
  {
    id: 'nextTab',
    name: 'Next Tab',
    description: 'Switch focus to the next tab to the right.',
    category: 'Tabs',
    defaultKey: 'Ctrl+Tab',
    defaultKeyMac: '⌃Tab'
  },
  {
    id: 'prevTab',
    name: 'Previous Tab',
    description: 'Switch focus to the previous tab to the left.',
    category: 'Tabs',
    defaultKey: 'Ctrl+Shift+Tab',
    defaultKeyMac: '⌃⇧Tab'
  },
  {
    id: 'toggleSidebar',
    name: 'Toggle Sidebar',
    description: 'Expand or collapse the primary navigation sidebar.',
    category: 'Workspace',
    defaultKey: 'Ctrl+B',
    defaultKeyMac: '⌘B'
  },
  {
    id: 'toggleTerminal',
    name: 'Toggle Integrated Terminal',
    description: 'Show or hide the integrated terminal panel.',
    category: 'Tools',
    defaultKey: 'Ctrl+`',
    defaultKeyMac: '⌘`'
  },
  {
    id: 'openSettings',
    name: 'Open Settings',
    description: 'Navigate to application preferences and settings.',
    category: 'Workspace',
    defaultKey: 'Ctrl+,',
    defaultKeyMac: '⌘,'
  }
] as const

export const DEFAULT_SHORTCUT_BINDINGS: Readonly<ShortcutBindings> = Object.freeze(
  DEFAULT_SHORTCUT_DEFINITIONS.reduce((acc, def) => {
    acc[def.id] = def.defaultKey
    return acc
  }, {} as ShortcutBindings)
)

export const DEFAULT_SHORTCUT_BINDINGS_MAC: Readonly<ShortcutBindings> = Object.freeze(
  DEFAULT_SHORTCUT_DEFINITIONS.reduce((acc, def) => {
    acc[def.id] = def.defaultKeyMac
    return acc
  }, {} as ShortcutBindings)
)

export function getDefaultBindingsForPlatform(isMac: boolean): ShortcutBindings {
  return isMac ? { ...DEFAULT_SHORTCUT_BINDINGS_MAC } : { ...DEFAULT_SHORTCUT_BINDINGS }
}
