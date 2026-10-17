import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  DEFAULT_SHORTCUT_DEFINITIONS,
  getDefaultBindingsForPlatform
} from '../constants/shortcuts.ts'
import { shortcutService } from '../services/shortcutService.ts'
import type { ShortcutActionId, ShortcutBindings } from '../types/shortcuts.ts'
import {
  findShortcutConflict,
  formatKeyCombo,
  isKeyComboMatch,
  normalizeComboString,
  parseKeyCombo
} from '../utils/shortcutHelpers.ts'
import type { KeyboardEventLike } from '../utils/keyboard.ts'

export interface PendingConflict {
  actionId: ShortcutActionId
  conflictingActionId: ShortcutActionId
  targetKey: string
  existingKey: string
}

export const useShortcutStore = defineStore('shortcut', () => {
  const isMac =
    typeof navigator !== 'undefined' &&
    Boolean(navigator.platform && navigator.platform.toUpperCase().includes('MAC'))

  const bindings = ref<ShortcutBindings>(getDefaultBindingsForPlatform(isMac))
  const isInitialized = ref(false)
  const isListening = ref(false)
  const listeningActionId = ref<ShortcutActionId | null>(null)
  const pendingConflict = ref<PendingConflict | null>(null)

  const definitions = computed(() => DEFAULT_SHORTCUT_DEFINITIONS)

  const hasAnyCustomizations = computed(() => {
    const defaults = getDefaultBindingsForPlatform(isMac)
    return Object.keys(bindings.value).some((key) => {
      const actId = key as ShortcutActionId
      return bindings.value[actId] !== defaults[actId]
    })
  })

  function isModified(actionId: ShortcutActionId): boolean {
    const defaults = getDefaultBindingsForPlatform(isMac)
    return bindings.value[actionId] !== defaults[actionId]
  }

  function getDisplayShortcut(actionId: ShortcutActionId): string {
    const raw = bindings.value[actionId]
    if (!raw) return ''
    const combo = parseKeyCombo(raw)
    return formatKeyCombo(combo, isMac)
  }

  async function init(overrideIsMac?: boolean): Promise<void> {
    const targetIsMac = overrideIsMac ?? isMac
    const loaded = await shortcutService.loadBindings(targetIsMac)
    bindings.value = loaded
    isInitialized.value = true
  }

  function startListening(actionId: ShortcutActionId): void {
    listeningActionId.value = actionId
    isListening.value = true
  }

  function stopListening(): void {
    listeningActionId.value = null
    isListening.value = false
  }

  async function rebind(
    actionId: ShortcutActionId,
    newComboString: string,
    overrideIsMac?: boolean
  ): Promise<{ success: boolean; conflictWith?: ShortcutActionId }> {
    const targetIsMac = overrideIsMac ?? isMac
    const normalized = normalizeComboString(newComboString, targetIsMac)
    if (!normalized) {
      stopListening()
      return { success: false }
    }

    const conflict = findShortcutConflict(actionId, normalized, bindings.value, targetIsMac)
    if (conflict) {
      pendingConflict.value = {
        actionId,
        conflictingActionId: conflict,
        targetKey: normalized,
        existingKey: bindings.value[actionId]
      }
      stopListening()
      return { success: false, conflictWith: conflict }
    }

    bindings.value = {
      ...bindings.value,
      [actionId]: normalized
    }

    await shortcutService.saveBindings(bindings.value, targetIsMac)
    stopListening()
    return { success: true }
  }

  async function confirmConflictSwap(overrideIsMac?: boolean): Promise<void> {
    if (!pendingConflict.value) return
    const targetIsMac = overrideIsMac ?? isMac
    const { actionId, conflictingActionId, targetKey, existingKey } = pendingConflict.value

    bindings.value = {
      ...bindings.value,
      [actionId]: targetKey,
      [conflictingActionId]: existingKey
    }

    await shortcutService.saveBindings(bindings.value, targetIsMac)
    pendingConflict.value = null
  }

  function cancelConflict(): void {
    pendingConflict.value = null
  }

  async function resetAction(actionId: ShortcutActionId, overrideIsMac?: boolean): Promise<void> {
    const targetIsMac = overrideIsMac ?? isMac
    const defaults = getDefaultBindingsForPlatform(targetIsMac)
    bindings.value = {
      ...bindings.value,
      [actionId]: defaults[actionId]
    }
    await shortcutService.saveBindings(bindings.value, targetIsMac)
  }

  async function resetAll(overrideIsMac?: boolean): Promise<void> {
    const targetIsMac = overrideIsMac ?? isMac
    const defaults = await shortcutService.resetBindings(targetIsMac)
    bindings.value = defaults
  }

  function matchesAction(
    event: KeyboardEventLike,
    actionId: ShortcutActionId,
    overrideIsMac?: boolean
  ): boolean {
    const targetIsMac = overrideIsMac ?? isMac
    const combo = bindings.value[actionId]
    if (!combo) return false
    return isKeyComboMatch(event, combo, targetIsMac)
  }

  return {
    bindings,
    definitions,
    isInitialized,
    isListening,
    listeningActionId,
    pendingConflict,
    hasAnyCustomizations,
    isModified,
    getDisplayShortcut,
    init,
    startListening,
    stopListening,
    rebind,
    confirmConflictSwap,
    cancelConflict,
    resetAction,
    resetAll,
    matchesAction
  }
})
