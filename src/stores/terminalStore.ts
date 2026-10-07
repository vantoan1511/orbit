import type { TerminalSession } from '../types/terminal.ts'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useTerminalStore = defineStore('terminal', () => {
  const isOpen = ref(false)
  const sessions = ref<TerminalSession[]>([])
  const activeSessionId = ref<string | null>(null)

  const activeSession = computed<TerminalSession | undefined>(() =>
    sessions.value.find((s) => s.id === activeSessionId.value)
  )

  function openPanel() {
    isOpen.value = true
  }

  function closePanel() {
    isOpen.value = false
  }

  function togglePanel() {
    isOpen.value = !isOpen.value
  }

  function addSession(session: TerminalSession) {
    sessions.value.push(session)
    activeSessionId.value = session.id
    isOpen.value = true
  }

  function removeSession(id: string) {
    const index = sessions.value.findIndex((s) => s.id === id)
    if (index === -1) return

    const isCurrentActive = activeSessionId.value === id

    sessions.value = sessions.value.filter((s) => s.id !== id)

    if (sessions.value.length === 0) {
      activeSessionId.value = null
      isOpen.value = false
    } else if (isCurrentActive) {
      // Pick the adjacent session (previous if possible, otherwise first)
      const nextIndex = Math.max(0, index - 1)
      activeSessionId.value = sessions.value[nextIndex]?.id ?? null
    }
  }

  function setActiveSession(id: string) {
    if (sessions.value.some((s) => s.id === id)) {
      activeSessionId.value = id
    }
  }

  return {
    isOpen,
    sessions,
    activeSessionId,
    activeSession,
    openPanel,
    closePanel,
    togglePanel,
    addSession,
    removeSession,
    setActiveSession
  }
})
