import { ref, computed } from 'vue'
import { storage } from '../services/nativeService.ts'

export interface HighlightRule {
  id: string
  pattern: string
  color: string
  bold: boolean
  caseSensitive: boolean
  isRegex: boolean
  enabled?: boolean
  isPreset?: boolean
}

export interface RuleValidationResult {
  valid: boolean
  error?: string
}

export type HighlightRuleUpdate = Partial<Omit<HighlightRule, 'id' | 'isPreset'>>

export function validateRule(
  pattern: string,
  isRegex: boolean,
  caseSensitive: boolean
): RuleValidationResult {
  const trimmed = pattern.trim()
  if (!trimmed) {
    return { valid: false, error: 'Pattern cannot be empty' }
  }

  if (isRegex) {
    try {
      new RegExp(trimmed, caseSensitive ? '' : 'i')
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Invalid regular expression'
      return { valid: false, error: message }
    }
  }

  return { valid: true }
}

export const PRESETS: Record<string, HighlightRule[]> = {
  standard: [
    {
      id: 'p-std-1',
      pattern: '\\b(error|fail|severe|fatal)\\b',
      color: 'rose',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-std-2',
      pattern: '\\b(warn|warning)\\b',
      color: 'amber',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-std-3',
      pattern: '\\binfo\\b',
      color: 'emerald',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-std-4',
      pattern: '\\bdebug\\b',
      color: 'emerald',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    }
  ],
  java: [
    {
      id: 'p-java-1',
      pattern: '\\b(ERROR|FATAL|SEVERE)\\b',
      color: 'rose',
      bold: true,
      caseSensitive: true,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-java-2',
      pattern: '\\b(WARN|WARNING)\\b',
      color: 'amber',
      bold: true,
      caseSensitive: true,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-java-3',
      pattern: '\\b(INFO|CONFIG)\\b',
      color: 'emerald',
      bold: true,
      caseSensitive: true,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-java-4',
      pattern: '\\b(DEBUG|FINE|FINER|FINEST)\\b',
      color: 'emerald',
      bold: true,
      caseSensitive: true,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-java-5',
      pattern: '\\bTRACE\\b',
      color: 'gray',
      bold: false,
      caseSensitive: true,
      isRegex: true,
      isPreset: true
    }
  ],
  go: [
    {
      id: 'p-go-1',
      pattern: '\\b(panic|fatal|error)\\b',
      color: 'rose',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-go-2',
      pattern: '\\b(warning|warn)\\b',
      color: 'amber',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-go-3',
      pattern: '\\binfo\\b',
      color: 'emerald',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-go-4',
      pattern: '\\bdebug\\b',
      color: 'emerald',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    }
  ],
  json: [
    {
      id: 'p-json-1',
      pattern: '"(level|lvl)"\\s*:\\s*"(error|fatal|panic)"',
      color: 'rose',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-json-2',
      pattern: '"(level|lvl)"\\s*:\\s*"(warn|warning)"',
      color: 'amber',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-json-3',
      pattern: '"(level|lvl)"\\s*:\\s*"(info|information)"',
      color: 'emerald',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    },
    {
      id: 'p-json-4',
      pattern: '"(level|lvl)"\\s*:\\s*"(debug|trace)"',
      color: 'emerald',
      bold: true,
      caseSensitive: false,
      isRegex: true,
      isPreset: true
    }
  ]
}

export function useLogHighlighting() {
  const showRulesDialog = ref(false)
  const selectedPreset = ref<string>('standard')
  const customRules = ref<HighlightRule[]>([])
  const customPresets = ref<Record<string, HighlightRule[]>>({})
  const newPresetName = ref<string>('')

  const presetOptions = computed(() => {
    const options = [
      {
        label: 'Standard Presets',
        items: [
          { label: 'Standard', value: 'standard' },
          { label: 'Java (SLF4J/Logback)', value: 'java' },
          { label: 'Go (Logrus/Zap)', value: 'go' },
          { label: 'JSON Logs', value: 'json' },
          { label: 'None', value: 'none' }
        ]
      }
    ]

    const customItems = Object.keys(customPresets.value).map((name) => ({
      label: name,
      value: `custom:${name}`
    }))

    if (customItems.length > 0) {
      options.push({
        label: 'Custom Presets',
        items: customItems
      })
    }

    return options
  })

  const activeRules = computed<HighlightRule[]>(() => {
    let preset: HighlightRule[] = []
    if (selectedPreset.value.startsWith('custom:')) {
      const presetName = selectedPreset.value.replace('custom:', '')
      preset = customPresets.value[presetName] || []
    } else {
      preset = PRESETS[selectedPreset.value] || []
    }
    return [...preset, ...customRules.value]
  })

  const isCustomPresetActive = computed(() => {
    return selectedPreset.value.startsWith('custom:')
  })

  const colorOptions = [
    { label: 'Red', value: 'rose' },
    { label: 'Orange', value: 'amber' },
    { label: 'Green', value: 'emerald' },
    { label: 'Blue', value: 'sky' },
    { label: 'Purple', value: 'purple' },
    { label: 'Pink', value: 'fuchsia' },
    { label: 'Gray', value: 'gray' }
  ]

  const loadRules = async () => {
    try {
      const savedCustomPresets = await storage.getData('orbit_log_custom_presets').catch(() => null)
      if (savedCustomPresets) {
        customPresets.value = JSON.parse(savedCustomPresets)
      } else {
        customPresets.value = {}
      }

      const savedPreset = await storage.getData('orbit_log_highlight_preset').catch(() => null)
      if (savedPreset) {
        selectedPreset.value = savedPreset
      } else {
        selectedPreset.value = 'standard'
      }

      const savedRules = await storage.getData('orbit_log_highlight_rules').catch(() => null)
      if (savedRules) {
        customRules.value = JSON.parse(savedRules)
      } else {
        customRules.value = []
      }
    } catch {
      selectedPreset.value = 'standard'
      customRules.value = []
      customPresets.value = {}
    }
  }

  const saveRules = async () => {
    try {
      await storage.setData('orbit_log_highlight_preset', selectedPreset.value)
      await storage.setData('orbit_log_highlight_rules', JSON.stringify(customRules.value))
      await storage.setData('orbit_log_custom_presets', JSON.stringify(customPresets.value))
    } catch (err) {
      console.error('Failed to save log highlight rules', err)
    }
  }

  const addRule = () => {
    customRules.value.push({
      id: Date.now().toString(),
      pattern: '',
      color: 'rose',
      bold: false,
      caseSensitive: false,
      isRegex: false
    })
    saveRules()
  }

  const isBuiltInRule = (rule: HighlightRule): boolean => {
    return Object.values(PRESETS).some((presetList) =>
      presetList.some((builtin) => builtin.id === rule.id)
    )
  }

  const updateCustomRule = (id: string, updates: HighlightRuleUpdate): boolean => {
    let targetRule: HighlightRule | undefined

    if (selectedPreset.value.startsWith('custom:')) {
      const presetName = selectedPreset.value.replace('custom:', '')
      const presetRules = customPresets.value[presetName]
      if (presetRules) {
        targetRule = presetRules.find((r) => r.id === id)
      }
    }

    if (!targetRule) {
      targetRule = customRules.value.find((r) => r.id === id)
    }

    if (!targetRule) {
      return false
    }

    const pattern = updates.pattern !== undefined ? updates.pattern : targetRule.pattern
    const isRegex = updates.isRegex !== undefined ? updates.isRegex : targetRule.isRegex
    const caseSensitive =
      updates.caseSensitive !== undefined ? updates.caseSensitive : targetRule.caseSensitive

    const validation = validateRule(pattern, isRegex, caseSensitive)
    if (!validation.valid) {
      return false
    }

    Object.assign(targetRule, updates)
    saveRules()
    return true
  }

  const deleteCustomRule = (id: string) => {
    if (selectedPreset.value.startsWith('custom:')) {
      const presetName = selectedPreset.value.replace('custom:', '')
      const presetRules = customPresets.value[presetName]
      if (presetRules) {
        const index = presetRules.findIndex((r) => r.id === id)
        if (index !== -1) {
          presetRules.splice(index, 1)
          saveRules()
          return
        }
      }
    }

    const index = customRules.value.findIndex((r) => r.id === id)
    if (index !== -1) {
      customRules.value.splice(index, 1)
      saveRules()
    }
  }

  const saveCustomPreset = async () => {
    const name = newPresetName.value.trim()
    if (!name) return

    const rulesToSave = customRules.value.map((rule) => ({
      ...rule,
      isPreset: true
    }))

    customPresets.value[name] = rulesToSave
    selectedPreset.value = `custom:${name}`
    newPresetName.value = ''
    await saveRules()
  }

  const deleteCustomPreset = async () => {
    if (selectedPreset.value.startsWith('custom:')) {
      const name = selectedPreset.value.replace('custom:', '')
      delete customPresets.value[name]
      selectedPreset.value = 'standard'
      await saveRules()
    }
  }

  const reorderRules = (fromIndex: number, toIndex: number): boolean => {
    let list: HighlightRule[] | undefined

    if (selectedPreset.value.startsWith('custom:')) {
      const name = selectedPreset.value.replace('custom:', '')
      list = customPresets.value[name]
    } else {
      list = customRules.value
    }

    if (
      !list ||
      fromIndex < 0 ||
      fromIndex >= list.length ||
      toIndex < 0 ||
      toIndex >= list.length
    ) {
      return false
    }

    const [moved] = list.splice(fromIndex, 1)
    if (!moved) return false
    list.splice(toIndex, 0, moved)
    saveRules()
    return true
  }

  const toggleRule = (id: string): boolean => {
    let targetRule: HighlightRule | undefined

    if (selectedPreset.value.startsWith('custom:')) {
      const presetName = selectedPreset.value.replace('custom:', '')
      const presetRules = customPresets.value[presetName]
      if (presetRules) {
        targetRule = presetRules.find((r) => r.id === id)
      }
    }

    if (!targetRule) {
      targetRule = customRules.value.find((r) => r.id === id)
    }

    if (!targetRule) {
      return false
    }

    targetRule.enabled = targetRule.enabled === false ? true : false
    saveRules()
    return true
  }

  const clonePreset = (sourcePresetKey: string, newPresetName: string): boolean => {
    const trimmed = newPresetName.trim()
    if (!trimmed) return false

    let sourceRules: HighlightRule[] = []
    if (sourcePresetKey.startsWith('custom:')) {
      const name = sourcePresetKey.replace('custom:', '')
      sourceRules = customPresets.value[name] || []
    } else {
      sourceRules = PRESETS[sourcePresetKey] || []
    }

    const cloned = sourceRules.map((rule, idx) => ({
      ...rule,
      id: `cp-${Date.now()}-${idx}`,
      isPreset: true,
      enabled: rule.enabled !== false
    }))

    customPresets.value[trimmed] = cloned
    selectedPreset.value = `custom:${trimmed}`
    saveRules()
    return true
  }

  const exportRules = (): string => {
    return JSON.stringify(activeRules.value, null, 2)
  }

  const importRules = (rules: HighlightRule[]): boolean => {
    if (!Array.isArray(rules)) return false

    const validRules: HighlightRule[] = []
    for (const r of rules) {
      if (!r || typeof r.pattern !== 'string') continue
      const validation = validateRule(r.pattern, !!r.isRegex, !!r.caseSensitive)
      if (!validation.valid) continue

      validRules.push({
        id: r.id || `imp-${Date.now()}-${validRules.length}`,
        pattern: r.pattern.trim(),
        color: r.color || 'rose',
        bold: !!r.bold,
        caseSensitive: !!r.caseSensitive,
        isRegex: !!r.isRegex,
        enabled: r.enabled !== false,
        isPreset: !!r.isPreset
      })
    }

    if (validRules.length === 0) return false

    if (selectedPreset.value.startsWith('custom:')) {
      const presetName = selectedPreset.value.replace('custom:', '')
      customPresets.value[presetName] = validRules
    } else {
      customRules.value = validRules
    }

    saveRules()
    return true
  }

  const getLogLevelColor = (text: string) => {
    const matched = activeRules.value.find((rule) => {
      if (rule.enabled === false) return false
      if (!rule.pattern) return false
      if (rule.isRegex) {
        try {
          const flags = rule.caseSensitive ? '' : 'i'
          const regex = new RegExp(rule.pattern, flags)
          return regex.test(text)
        } catch {
          return false
        }
      } else {
        const matchText = rule.caseSensitive ? text : text.toLowerCase()
        const pattern = rule.caseSensitive ? rule.pattern : rule.pattern.toLowerCase()
        return matchText.includes(pattern)
      }
    })

    if (matched) {
      let classes = ''
      switch (matched.color) {
        case 'rose':
          classes = 'text-rose-500'
          break
        case 'amber':
          classes = 'text-amber-500'
          break
        case 'emerald':
          classes = 'text-emerald-500'
          break
        case 'sky':
          classes = 'text-sky-500'
          break
        case 'purple':
          classes = 'text-purple-500'
          break
        case 'fuchsia':
          classes = 'text-fuchsia-500'
          break
        case 'gray':
          classes = 'text-zinc-500 dark:text-zinc-400'
          break
        default:
          classes = 'text-muted-color'
          break
      }
      if (matched.bold) {
        classes += ' font-bold'
      }
      return classes
    }

    return 'text-muted-color'
  }

  return {
    showRulesDialog,
    selectedPreset,
    customRules,
    customPresets,
    newPresetName,
    presetOptions,
    activeRules,
    isCustomPresetActive,
    colorOptions,
    loadRules,
    saveRules,
    addRule,
    updateCustomRule,
    deleteCustomRule,
    saveCustomPreset,
    deleteCustomPreset,
    reorderRules,
    toggleRule,
    clonePreset,
    exportRules,
    importRules,
    isBuiltInRule,
    validateRule,
    getLogLevelColor
  }
}
