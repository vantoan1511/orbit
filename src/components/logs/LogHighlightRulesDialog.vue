<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { HighlightRule, useLogHighlighting } from '@/composables/useLogHighlighting'
import {
  AlertCircle,
  Check,
  ChevronDown,
  ChevronUp,
  Copy,
  Download,
  Plus,
  RotateCcw,
  Save,
  Trash2,
  Upload
} from '@lucide/vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import Dialog from 'primevue/dialog'
import IconField from 'primevue/iconfield'
import InputIcon from 'primevue/inputicon'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Textarea from 'primevue/textarea'
import ToggleSwitch from 'primevue/toggleswitch'

const props = withDefaults(
  defineProps<{
    highlighting: ReturnType<typeof useLogHighlighting>
    recentLogs?: string[]
  }>(),
  {
    recentLogs: () => []
  }
)

const visible = defineModel<boolean>('visible', { default: false })

// Presets state
const selectedPreset = props.highlighting.selectedPreset
const newPresetName = props.highlighting.newPresetName
const customRules = props.highlighting.customRules

// Master-detail state
const searchQuery = ref('')
const selectedRuleId = ref<string | null>(null)
const customTestLine = ref('')
type ScenarioType = 'microservice' | 'ingress' | 'json' | 'recent'
const activeScenario = ref<ScenarioType>('microservice')

// Sub-dialogs state
const isCloneModalOpen = ref(false)
const cloneNameInput = ref('')
const isImportModalOpen = ref(false)
const importJsonText = ref('')
const importError = ref<string | null>(null)

// Color palette options
const COLOR_PALETTE = [
  {
    name: 'rose',
    label: 'Red / Rose',
    class: 'text-rose-500',
    bgClass: 'bg-rose-500',
    hex: '#f43f5e'
  },
  {
    name: 'amber',
    label: 'Orange / Amber',
    class: 'text-amber-500',
    bgClass: 'bg-amber-500',
    hex: '#f59e0b'
  },
  {
    name: 'emerald',
    label: 'Green / Emerald',
    class: 'text-emerald-500',
    bgClass: 'bg-emerald-500',
    hex: '#10b981'
  },
  {
    name: 'sky',
    label: 'Blue / Sky',
    class: 'text-sky-500',
    bgClass: 'bg-sky-500',
    hex: '#0284c7'
  },
  {
    name: 'purple',
    label: 'Purple',
    class: 'text-purple-500',
    bgClass: 'bg-purple-500',
    hex: '#a855f7'
  },
  {
    name: 'fuchsia',
    label: 'Pink / Fuchsia',
    class: 'text-fuchsia-500',
    bgClass: 'bg-fuchsia-500',
    hex: '#d946ef'
  },
  {
    name: 'gray',
    label: 'Muted Zinc',
    class: 'text-zinc-500 dark:text-zinc-400',
    bgClass: 'bg-zinc-500',
    hex: '#71717a'
  }
] as const

const getColorOption = (name: string) => {
  return COLOR_PALETTE.find((c) => c.name === name) ?? COLOR_PALETTE[0]!
}

// Built-in sample stream scenarios for testing
const SAMPLE_LOGS: Record<'microservice' | 'ingress' | 'json', string[]> = {
  microservice: [
    '2026-10-14T15:20:01.002Z [auth-service/api-6bf9c] [INFO] Worker pool initialized with 16 go-routines',
    '2026-10-14T15:20:01.140Z [auth-service/api-6bf9c] [DEBUG] Syncing OAuth state token for client 849a-01',
    '2026-10-14T15:20:02.503Z [auth-service/api-6bf9c] [WARN] Slow database query detected on users_by_email duration=412ms',
    '2026-10-14T15:20:03.882Z [auth-service/api-6bf9c] [ERROR] Failed to connect to Redis replica at 10.244.2.14:6379 - connection refused',
    '2026-10-14T15:20:04.102Z [auth-service/api-6bf9c] [INFO] Fallback to primary in-memory cache succeeded',
    '2026-10-14T15:20:05.912Z [auth-service/api-6bf9c] [FATAL] Critical panic in session decryptor: runtime error invalid memory address'
  ],
  ingress: [
    '10.244.0.1 - - [14/Oct/2026:15:20:01 +0000] "GET /api/v1/healthz HTTP/1.1" 200 14 "-" "kube-probe/1.28"',
    '10.244.0.54 - - [14/Oct/2026:15:20:02 +0000] "GET /api/v2/orders/4081 HTTP/2.0" 200 1824 "-" "Mozilla/5.0"',
    '192.168.1.109 - - [14/Oct/2026:15:20:03 +0000] "POST /api/v1/checkout HTTP/2.0" 404 128 "-" "MobileApp/2.4"',
    '172.16.4.22 - - [14/Oct/2026:15:20:04 +0000] "PUT /api/v1/billing HTTP/1.1" 500 892 "-" "StripeWebhook/1.0"',
    '10.244.1.88 - - [14/Oct/2026:15:20:05 +0000] "GET /metrics HTTP/1.1" 200 4092 "-" "Prometheus/2.45"'
  ],
  json: [
    '{"timestamp":"2026-10-14T15:20:01Z","level":"info","caller":"server.go:88","msg":"Listening on :8080"}',
    '{"timestamp":"2026-10-14T15:20:02Z","level":"debug","caller":"auth.go:42","msg":"JWT validated successfully","sub":"usr-992"}',
    '{"timestamp":"2026-10-14T15:20:03Z","level":"warn","caller":"circuit.go:110","msg":"Degraded circuit breaker for payments service"}',
    '{"timestamp":"2026-10-14T15:20:04Z","level":"error","caller":"db.go:214","msg":"Deadlock detected during transaction 98401"}',
    '{"timestamp":"2026-10-14T15:20:05Z","level":"panic","caller":"main.go:301","msg":"Uncaught nil pointer exception in background processor"}'
  ]
}

// Filtered rules in current pipeline
const filteredRules = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return props.highlighting.activeRules.value
  return props.highlighting.activeRules.value.filter((r) => r.pattern.toLowerCase().includes(query))
})

// Current selected rule
const selectedRule = computed<HighlightRule | undefined>(() => {
  if (selectedRuleId.value) {
    const found = props.highlighting.activeRules.value.find((r) => r.id === selectedRuleId.value)
    if (found) return found
  }
  return props.highlighting.activeRules.value[0]
})

// Ensure a rule is always selected
watch(
  () => props.highlighting.activeRules.value,
  (rules) => {
    if (
      rules.length > 0 &&
      rules[0] &&
      (!selectedRuleId.value || !rules.some((r) => r.id === selectedRuleId.value))
    ) {
      selectedRuleId.value = rules[0].id
    }
  },
  { immediate: true }
)

const isSelectedBuiltIn = computed(() => {
  return selectedRule.value ? props.highlighting.isBuiltInRule(selectedRule.value) : false
})

const selectedRuleIndex = computed(() => {
  if (!selectedRule.value) return -1
  return props.highlighting.activeRules.value.findIndex((r) => r.id === selectedRule.value!.id)
})

// Real-time syntax validation for selected rule
const ruleValidationError = computed(() => {
  if (!selectedRule.value) return null
  if (!selectedRule.value.pattern.trim()) return 'Pattern cannot be empty'
  const validation = props.highlighting.validateRule(
    selectedRule.value.pattern,
    selectedRule.value.isRegex,
    selectedRule.value.caseSensitive
  )
  return validation.valid ? null : (validation.error ?? 'Invalid regular expression')
})

// Actions
const selectRule = (id: string) => {
  selectedRuleId.value = id
}

const updateCurrentRuleField = (field: keyof HighlightRule, value: unknown) => {
  if (!selectedRule.value || isSelectedBuiltIn.value) return
  props.highlighting.updateCustomRule(selectedRule.value.id, { [field]: value })
}

const handleMoveSelected = (direction: -1 | 1) => {
  const currentIdx = selectedRuleIndex.value
  if (currentIdx === -1) return
  const targetIdx = currentIdx + direction
  const success = props.highlighting.reorderRules(currentIdx, targetIdx)
  if (success && selectedRule.value) {
    selectedRuleId.value = selectedRule.value.id
  }
}

const handleDeleteSelected = () => {
  if (!selectedRule.value || isSelectedBuiltIn.value) return
  const currentId = selectedRule.value.id
  props.highlighting.deleteCustomRule(currentId)
  nextTick(() => {
    const remaining = props.highlighting.activeRules.value
    if (remaining.length > 0 && remaining[0]) {
      selectedRuleId.value = remaining[0].id
    }
  })
}

const handleAddRule = () => {
  props.highlighting.addRule()
  nextTick(() => {
    const rules = props.highlighting.activeRules.value
    const last = rules[rules.length - 1]
    if (last) {
      selectedRuleId.value = last.id
    }
  })
}

const addQuickSnippet = (type: 'http_error' | 'ip_address' | 'panic_trace' | 'oom_kill') => {
  const snippets = {
    http_error: { pattern: '\\b(4\\d{2}|5\\d{2})\b', color: 'rose', bold: true, isRegex: true },
    ip_address: {
      pattern: '\\b\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\b',
      color: 'sky',
      bold: false,
      isRegex: true
    },
    panic_trace: {
      pattern: '\\b(panic:|goroutine \\d+|stack backtrace:)',
      color: 'fuchsia',
      bold: true,
      isRegex: true
    },
    oom_kill: {
      pattern: '\\b(OOMKilled|exit code 137|out of memory)\\b',
      color: 'purple',
      bold: true,
      isRegex: true
    }
  }
  const s = snippets[type]
  if (!s) return

  const newId = `rule-${Date.now()}`
  customRules.value.unshift({
    id: newId,
    pattern: s.pattern,
    color: s.color,
    bold: s.bold,
    caseSensitive: false,
    isRegex: s.isRegex,
    enabled: true,
    isPreset: false
  })
  props.highlighting.saveRules()
  selectedRuleId.value = newId
}

// Preset clone & JSON export/import handlers
const openCloneModal = () => {
  cloneNameInput.value = `${selectedPreset.value.replace('custom:', '')} Copy`
  isCloneModalOpen.value = true
}

const confirmClone = () => {
  const name = cloneNameInput.value.trim()
  if (!name) return
  props.highlighting.clonePreset(selectedPreset.value, name)
  isCloneModalOpen.value = false
}

const handleExportJson = () => {
  const jsonStr = props.highlighting.exportRules()
  const blob = new Blob([jsonStr], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `orbit_highlight_rules_${selectedPreset.value.replace(/[^a-zA-Z0-9_-]/g, '_')}.json`
  a.click()
  URL.revokeObjectURL(url)
}

const openImportModal = () => {
  importJsonText.value = ''
  importError.value = null
  isImportModalOpen.value = true
}

const confirmImport = () => {
  importError.value = null
  try {
    const parsed = JSON.parse(importJsonText.value)
    if (!Array.isArray(parsed)) {
      importError.value = 'Expected a JSON array of highlight rules'
      return
    }
    const success = props.highlighting.importRules(parsed)
    if (success) {
      isImportModalOpen.value = false
    } else {
      importError.value = 'No valid highlight rules found in imported data'
    }
  } catch (err) {
    importError.value = err instanceof Error ? err.message : 'Invalid JSON format'
  }
}

const revertToDefaults = () => {
  selectedPreset.value = 'standard'
  props.highlighting.saveRules()
}

// Live preview simulation lines computation
const previewLines = computed<string[]>(() => {
  let base: string[] = []
  if (activeScenario.value === 'recent' && props.recentLogs.length > 0) {
    base = [...props.recentLogs]
  } else if (activeScenario.value === 'recent') {
    base = SAMPLE_LOGS.microservice
  } else {
    base = [...SAMPLE_LOGS[activeScenario.value]]
  }

  if (customTestLine.value.trim()) {
    base.unshift(customTestLine.value.trim())
  }
  return base
})

interface MatchedLineResult {
  text: string
  rule: HighlightRule | null
  colorClass: string
  bold: boolean
  highlightHtml: string
}

const evaluatedPreviewLines = computed<MatchedLineResult[]>(() => {
  const activeRules = props.highlighting.activeRules.value

  return previewLines.value.map((line) => {
    let matchedRule: HighlightRule | null = null

    for (const rule of activeRules) {
      if (rule.enabled === false || !rule.pattern) continue

      if (rule.isRegex) {
        try {
          const flags = rule.caseSensitive ? '' : 'i'
          const regex = new RegExp(rule.pattern, flags)
          if (regex.test(line)) {
            matchedRule = rule
            break
          }
        } catch {
          // ignore regex errors in preview
        }
      } else {
        const matchText = rule.caseSensitive ? line : line.toLowerCase()
        const pattern = rule.caseSensitive ? rule.pattern : rule.pattern.toLowerCase()
        if (matchText.includes(pattern)) {
          matchedRule = rule
          break
        }
      }
    }

    if (!matchedRule) {
      return {
        text: line,
        rule: null,
        colorClass: 'text-zinc-400',
        bold: false,
        highlightHtml: escapeHtml(line)
      }
    }

    const colorObj = getColorOption(matchedRule.color)
    let highlightHtml = escapeHtml(line)

    try {
      const flags = matchedRule.caseSensitive ? 'g' : 'gi'
      const regex = matchedRule.isRegex
        ? new RegExp(`(${matchedRule.pattern})`, flags)
        : new RegExp(`(${escapeRegex(matchedRule.pattern)})`, flags)
      highlightHtml = escapeHtml(line).replace(
        regex,
        `<mark class="bg-white/20 text-inherit px-0.5 rounded underline decoration-white/30">$1</mark>`
      )
    } catch {
      highlightHtml = escapeHtml(line)
    }

    return {
      text: line,
      rule: matchedRule,
      colorClass: colorObj.class,
      bold: matchedRule.bold,
      highlightHtml
    }
  })
})

const matchedCount = computed(() => {
  return evaluatedPreviewLines.value.filter((l) => l.rule !== null).length
})

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

function escapeRegex(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    header="Log Highlight Rules Studio"
    :style="{ width: '1040px', maxWidth: '96vw' }"
  >
    <div class="flex flex-col gap-4">
      <!-- Preset Management Toolbar (Zonal Tone Shift) -->
      <div
        class="bg-(--bg-hover)/40 rounded-lg p-3 flex flex-wrap items-center justify-between gap-3"
      >
        <div class="flex items-center gap-2.5">
          <span class="text-[10px] font-semibold uppercase tracking-wider text-muted-color"
            >Preset:</span
          >
          <Select
            v-model="selectedPreset"
            :options="highlighting.presetOptions.value"
            optionLabel="label"
            optionValue="value"
            optionGroupLabel="label"
            optionGroupChildren="items"
            size="small"
            class="text-xs min-w-52"
            @change="highlighting.saveRules"
          />
          <Button
            size="small"
            severity="secondary"
            variant="outlined"
            class="text-xs"
            v-tooltip.top="'Clone preset as an editable custom configuration'"
            @click="openCloneModal"
          >
            <template #icon>
              <Copy class="w-3.5 h-3.5 mr-1" />
            </template>
            Clone
          </Button>
          <Button
            v-if="highlighting.isCustomPresetActive.value"
            severity="danger"
            variant="text"
            size="small"
            class="w-8! h-8! p-0! shrink-0"
            v-tooltip.top="'Delete active custom preset'"
            @click="highlighting.deleteCustomPreset"
          >
            <template #icon>
              <Trash2 class="w-4 h-4" />
            </template>
          </Button>
        </div>

        <div class="flex items-center gap-2">
          <Button
            size="small"
            severity="secondary"
            variant="text"
            class="text-xs"
            v-tooltip.top="'Export active rules as JSON'"
            @click="handleExportJson"
          >
            <template #icon>
              <Download class="w-3.5 h-3.5 mr-1" />
            </template>
            Export
          </Button>
          <Button
            size="small"
            severity="secondary"
            variant="text"
            class="text-xs"
            v-tooltip.top="'Import rules from JSON'"
            @click="openImportModal"
          >
            <template #icon>
              <Upload class="w-3.5 h-3.5 mr-1" />
            </template>
            Import
          </Button>
          <span class="w-px h-4 bg-(--border)"></span>
          <InputText
            v-model="newPresetName"
            placeholder="Save as new preset..."
            size="small"
            class="text-xs w-44"
          />
          <Button
            label="Save"
            size="small"
            severity="secondary"
            :disabled="!highlighting.customRules.value.length || !newPresetName.trim()"
            @click="highlighting.saveCustomPreset"
          >
            <template #icon>
              <Save class="w-3.5 h-3.5 mr-1" />
            </template>
          </Button>
        </div>
      </div>

      <!-- Master-Detail Body Studio -->
      <div class="flex flex-col md:flex-row gap-4 h-[540px] overflow-hidden">
        <!-- LEFT PANE: Master Rules Pipeline -->
        <div
          class="w-full md:w-84 lg:w-96 flex flex-col border border-(--border) rounded-lg bg-(--bg-card) overflow-hidden shrink-0"
        >
          <!-- Master Header & Search Filter -->
          <div
            class="p-2.5 border-b border-(--border) flex items-center justify-between gap-2 bg-(--bg-sidebar)"
          >
            <IconField class="flex-1">
              <InputIcon class="pi pi-search text-xs" />
              <InputText
                v-model="searchQuery"
                placeholder="Filter rules..."
                size="small"
                fluid
                class="text-xs"
              />
            </IconField>
            <Button
              size="small"
              class="text-xs shrink-0"
              v-tooltip.top="'Add a new custom highlight rule'"
              @click="handleAddRule"
            >
              <template #icon>
                <Plus class="w-3.5 h-3.5 mr-1" />
              </template>
              New Rule
            </Button>
          </div>

          <!-- Precedence Subheader -->
          <div
            class="px-3 py-1.5 bg-(--bg-panel) border-b border-(--border) flex items-center justify-between text-[10px] text-muted-color select-none"
          >
            <span class="font-semibold tracking-wider uppercase"
              >Order (Evaluated Top-to-Bottom)</span
            >
            <span>{{ filteredRules.length }} rules</span>
          </div>

          <!-- Rules List -->
          <div class="flex-1 overflow-y-auto divide-y divide-(--border)/60 pr-0.5">
            <div v-if="filteredRules.length === 0" class="p-6 text-center text-xs text-muted-color">
              No rules matching filter.
            </div>
            <div
              v-for="(rule, idx) in filteredRules"
              :key="rule.id"
              class="p-2 px-3 flex items-center justify-between gap-2.5 cursor-pointer transition-colors group"
              :class="[
                selectedRule?.id === rule.id
                  ? 'bg-(--bg-hover) border-l-2 border-(--accent)'
                  : 'hover:bg-(--bg-hover)/60'
              ]"
              @click="selectRule(rule.id)"
            >
              <!-- Index + Swatch + Pattern -->
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <span
                  class="text-[11px] font-mono text-(--text-disabled) w-4 text-right select-none"
                >
                  {{ idx + 1 }}
                </span>
                <span
                  class="w-2.5 h-2.5 rounded-full shrink-0"
                  :style="{ backgroundColor: getColorOption(rule.color).hex }"
                ></span>
                <div class="flex flex-col min-w-0 flex-1">
                  <span
                    class="font-mono text-xs truncate"
                    :class="[
                      rule.enabled === false
                        ? 'line-through text-(--text-disabled)'
                        : getColorOption(rule.color).class,
                      rule.bold ? 'font-bold' : ''
                    ]"
                  >
                    {{ rule.pattern || '(empty pattern)' }}
                  </span>
                  <div class="flex items-center gap-1.5 mt-0.5 text-[10px] text-muted-color">
                    <span
                      v-if="rule.isRegex"
                      class="px-1 rounded bg-(--bg-card) border border-(--border) font-mono text-[9px]"
                      >Rx</span
                    >
                    <span
                      v-if="rule.caseSensitive"
                      class="px-1 rounded bg-(--bg-card) border border-(--border) font-mono text-[9px]"
                      >Aa</span
                    >
                    <span
                      v-if="highlighting.isBuiltInRule(rule)"
                      class="text-[9px] text-(--text-disabled)"
                      >Preset</span
                    >
                    <span v-else class="text-[9px] text-emerald-500 font-medium">Custom</span>
                  </div>
                </div>
              </div>

              <!-- Enable Switch -->
              <div class="shrink-0 flex items-center" @click.stop>
                <ToggleSwitch
                  :modelValue="rule.enabled !== false"
                  class="scale-75"
                  v-tooltip.top="
                    rule.enabled !== false
                      ? 'Rule active (click to mute)'
                      : 'Rule muted (click to enable)'
                  "
                  @update:modelValue="highlighting.toggleRule(rule.id)"
                />
              </div>
            </div>
          </div>

          <!-- Quick Token Snippets Footer -->
          <div class="p-2.5 border-t border-(--border) bg-(--bg-panel) flex flex-col gap-1.5">
            <span class="text-[10px] font-semibold text-muted-color uppercase tracking-wider"
              >Quick Snippets:</span
            >
            <div class="flex flex-wrap gap-1.5">
              <Button
                label="+ 4xx/5xx HTTP"
                size="small"
                severity="secondary"
                variant="outlined"
                class="text-[11px]! py-0.5! px-2!"
                @click="addQuickSnippet('http_error')"
              />
              <Button
                label="+ IPv4 Addr"
                size="small"
                severity="secondary"
                variant="outlined"
                class="text-[11px]! py-0.5! px-2!"
                @click="addQuickSnippet('ip_address')"
              />
              <Button
                label="+ Panic / Stack"
                size="small"
                severity="secondary"
                variant="outlined"
                class="text-[11px]! py-0.5! px-2!"
                @click="addQuickSnippet('panic_trace')"
              />
              <Button
                label="+ OOMKilled"
                size="small"
                severity="secondary"
                variant="outlined"
                class="text-[11px]! py-0.5! px-2!"
                @click="addQuickSnippet('oom_kill')"
              />
            </div>
          </div>
        </div>

        <!-- RIGHT PANE: Detail Inspector & Live Console Sandbox -->
        <div
          class="flex-1 flex flex-col border border-(--border) rounded-lg bg-(--bg-card) overflow-hidden min-w-0"
        >
          <!-- TOP: Rule Inspector -->
          <div
            class="p-4 border-b border-(--border) bg-(--bg-sidebar) flex flex-col gap-3 shrink-0"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span
                  class="text-[10px] font-semibold px-2 py-0.5 rounded uppercase tracking-wider"
                  :class="[
                    isSelectedBuiltIn
                      ? 'bg-zinc-500/10 text-zinc-400'
                      : 'bg-emerald-500/10 text-emerald-500'
                  ]"
                >
                  {{ isSelectedBuiltIn ? 'Built-in Preset Rule (Read-only)' : 'Custom Rule' }}
                </span>
                <span class="text-xs font-mono text-muted-color">#{{ selectedRule?.id }}</span>
              </div>

              <!-- Reorder & Delete Actions -->
              <div class="flex items-center gap-1">
                <Button
                  severity="secondary"
                  variant="text"
                  size="small"
                  class="w-7! h-7! p-0!"
                  :disabled="selectedRuleIndex <= 0"
                  v-tooltip.top="'Move up (higher precedence)'"
                  @click="handleMoveSelected(-1)"
                >
                  <template #icon>
                    <ChevronUp class="w-4 h-4" />
                  </template>
                </Button>
                <Button
                  severity="secondary"
                  variant="text"
                  size="small"
                  class="w-7! h-7! p-0!"
                  :disabled="
                    selectedRuleIndex < 0 ||
                    selectedRuleIndex >= highlighting.activeRules.value.length - 1
                  "
                  v-tooltip.top="'Move down (lower precedence)'"
                  @click="handleMoveSelected(1)"
                >
                  <template #icon>
                    <ChevronDown class="w-4 h-4" />
                  </template>
                </Button>
                <span class="w-px h-3.5 bg-(--border) mx-1"></span>
                <Button
                  severity="danger"
                  variant="text"
                  size="small"
                  class="w-7! h-7! p-0!"
                  :disabled="isSelectedBuiltIn || !selectedRule"
                  v-tooltip.top="'Delete rule'"
                  @click="handleDeleteSelected"
                >
                  <template #icon>
                    <Trash2 class="w-4 h-4" />
                  </template>
                </Button>
              </div>
            </div>

            <!-- Pattern Input -->
            <div class="flex flex-col gap-1.5" v-if="selectedRule">
              <div class="flex items-center justify-between">
                <label class="text-xs font-semibold text-muted-color">Pattern</label>
                <span
                  v-if="!ruleValidationError"
                  class="text-[11px] font-medium text-emerald-500 flex items-center gap-1"
                >
                  <Check class="w-3 h-3" />
                  Valid pattern
                </span>
                <span v-else class="text-[11px] font-medium text-rose-500 flex items-center gap-1">
                  <AlertCircle class="w-3 h-3" />
                  {{ ruleValidationError }}
                </span>
              </div>
              <InputText
                :modelValue="selectedRule.pattern"
                placeholder="e.g. error, timeout, \b(404|500)\b"
                size="small"
                class="font-mono text-xs w-full"
                :invalid="!!ruleValidationError"
                :disabled="isSelectedBuiltIn"
                @update:modelValue="updateCurrentRuleField('pattern', $event)"
              />
            </div>

            <!-- Match Flags & Color Swatches -->
            <div
              class="grid grid-cols-1 lg:grid-cols-2 gap-4 items-center pt-1"
              v-if="selectedRule"
            >
              <div class="flex items-center gap-4">
                <div class="flex items-center gap-1.5">
                  <Checkbox
                    :modelValue="selectedRule.isRegex"
                    :inputId="'rx-' + selectedRule.id"
                    binary
                    :disabled="isSelectedBuiltIn"
                    @update:modelValue="updateCurrentRuleField('isRegex', $event)"
                  />
                  <label
                    :for="'rx-' + selectedRule.id"
                    class="text-xs text-muted-color cursor-pointer select-none"
                    >Regex</label
                  >
                </div>
                <div class="flex items-center gap-1.5">
                  <Checkbox
                    :modelValue="selectedRule.caseSensitive"
                    :inputId="'cs-' + selectedRule.id"
                    binary
                    :disabled="isSelectedBuiltIn"
                    @update:modelValue="updateCurrentRuleField('caseSensitive', $event)"
                  />
                  <label
                    :for="'cs-' + selectedRule.id"
                    class="text-xs text-muted-color cursor-pointer select-none"
                    >Case Sensitive</label
                  >
                </div>
                <div class="flex items-center gap-1.5">
                  <Checkbox
                    :modelValue="selectedRule.bold"
                    :inputId="'bold-' + selectedRule.id"
                    binary
                    :disabled="isSelectedBuiltIn"
                    @update:modelValue="updateCurrentRuleField('bold', $event)"
                  />
                  <label
                    :for="'bold-' + selectedRule.id"
                    class="text-xs text-muted-color cursor-pointer select-none"
                    >Bold</label
                  >
                </div>
              </div>

              <!-- Color Palette -->
              <div class="flex items-center gap-2 justify-start lg:justify-end">
                <span class="text-xs text-muted-color">Color:</span>
                <div class="flex items-center gap-1.5">
                  <Button
                    v-for="color in COLOR_PALETTE"
                    :key="color.name"
                    size="small"
                    variant="text"
                    class="w-5! h-5! p-0! rounded-full shrink-0 border border-white/20 transition-transform"
                    :class="[
                      selectedRule.color === color.name ? 'ring-2 ring-(--accent) scale-110' : ''
                    ]"
                    :style="{ backgroundColor: color.hex }"
                    :disabled="isSelectedBuiltIn"
                    v-tooltip.top="color.label"
                    @click="updateCurrentRuleField('color', color.name)"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- BOTTOM: Live Console Sandbox -->
          <div class="flex-1 flex flex-col min-h-0 bg-[#08090a] text-zinc-300 font-mono text-xs">
            <!-- Sandbox Header -->
            <div
              class="px-3 py-2 border-b border-zinc-800 bg-[#0d0f12] flex items-center justify-between shrink-0"
            >
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                <span class="font-semibold text-xs tracking-wider uppercase text-zinc-300"
                  >Live Preview Console</span
                >
                <span class="text-zinc-600">|</span>
                <span class="text-[11px] text-zinc-400 font-sans">
                  {{ matchedCount }} of {{ previewLines.length }} lines matched
                </span>
              </div>

              <!-- Scenario switcher -->
              <div class="flex items-center gap-1">
                <Button
                  label="Microservice"
                  size="small"
                  :variant="activeScenario === 'microservice' ? 'primary' : 'text'"
                  severity="secondary"
                  class="text-[11px]! py-0.5! px-2!"
                  @click="activeScenario = 'microservice'"
                />
                <Button
                  label="Ingress"
                  size="small"
                  :variant="activeScenario === 'ingress' ? 'primary' : 'text'"
                  severity="secondary"
                  class="text-[11px]! py-0.5! px-2!"
                  @click="activeScenario = 'ingress'"
                />
                <Button
                  label="JSON"
                  size="small"
                  :variant="activeScenario === 'json' ? 'primary' : 'text'"
                  severity="secondary"
                  class="text-[11px]! py-0.5! px-2!"
                  @click="activeScenario = 'json'"
                />
                <Button
                  v-if="recentLogs.length > 0"
                  label="Live Cluster"
                  size="small"
                  :variant="activeScenario === 'recent' ? 'primary' : 'text'"
                  severity="secondary"
                  class="text-[11px]! py-0.5! px-2!"
                  v-tooltip.top="'Test against current active cluster log stream'"
                  @click="activeScenario = 'recent'"
                />
              </div>
            </div>

            <!-- Log Stream Display -->
            <div class="flex-1 overflow-y-auto p-3 space-y-1 select-text">
              <div
                v-for="(item, idx) in evaluatedPreviewLines"
                :key="idx"
                class="py-0.5 px-2 rounded flex items-start gap-2 transition-colors"
                :class="[
                  item.rule
                    ? 'bg-zinc-900/60 border-l-2 border-emerald-500/80 hover:bg-zinc-800/50'
                    : 'hover:bg-zinc-900/40'
                ]"
              >
                <span
                  class="text-[10px] select-none font-sans shrink-0 uppercase tracking-wider py-0.5"
                  :class="item.rule ? 'text-zinc-400 font-medium' : 'text-zinc-700'"
                >
                  [{{ item.rule ? item.rule.pattern.slice(0, 14) || 'MATCH' : 'DEFAULT' }}]
                </span>
                <span
                  class="flex-1 break-all"
                  :class="[item.colorClass, item.bold ? 'font-bold' : '']"
                  v-html="item.highlightHtml"
                ></span>
              </div>
            </div>

            <!-- Custom Test Line Box -->
            <div class="p-2 border-t border-zinc-800 bg-[#0d0f12] flex items-center gap-2 shrink-0">
              <span class="text-zinc-500 text-[10px] uppercase tracking-wider font-sans shrink-0"
                >Test Line:</span
              >
              <InputText
                v-model="customTestLine"
                placeholder="Type any test log string here to immediately preview matching..."
                size="small"
                fluid
                class="font-mono text-xs bg-zinc-900! border-zinc-800! text-zinc-200!"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Clone Preset Modal Dialog -->
    <Dialog
      v-model:visible="isCloneModalOpen"
      modal
      header="Clone Preset as Custom"
      :style="{ width: '420px', maxWidth: '92vw' }"
    >
      <div class="flex flex-col gap-2 py-2">
        <label class="text-xs font-semibold text-muted-color">New Custom Preset Name</label>
        <InputText
          v-model="cloneNameInput"
          placeholder="e.g. SRE Incident Preset"
          size="small"
          class="text-xs w-full"
          autofocus
        />
      </div>
      <template #footer>
        <div class="flex justify-end gap-2 w-full">
          <Button
            label="Cancel"
            severity="secondary"
            variant="text"
            size="small"
            @click="isCloneModalOpen = false"
          />
          <Button
            label="Clone Preset"
            size="small"
            :disabled="!cloneNameInput.trim()"
            @click="confirmClone"
          />
        </div>
      </template>
    </Dialog>

    <!-- Import JSON Modal Dialog -->
    <Dialog
      v-model:visible="isImportModalOpen"
      modal
      header="Import Rules from JSON"
      :style="{ width: '480px', maxWidth: '92vw' }"
    >
      <div class="flex flex-col gap-2 py-2">
        <label class="text-xs font-semibold text-muted-color">Paste JSON Rules Array</label>
        <Textarea
          v-model="importJsonText"
          rows="8"
          autoResize
          placeholder='[{"pattern": "\\b(error|fail)\\b", "color": "rose", "bold": true, "isRegex": true}]'
          class="w-full font-mono text-xs"
        />
        <span v-if="importError" class="text-xs text-rose-500">{{ importError }}</span>
      </div>
      <template #footer>
        <div class="flex justify-end gap-2 w-full">
          <Button
            label="Cancel"
            severity="secondary"
            variant="text"
            size="small"
            @click="isImportModalOpen = false"
          />
          <Button
            label="Import Rules"
            size="small"
            :disabled="!importJsonText.trim()"
            @click="confirmImport"
          />
        </div>
      </template>
    </Dialog>

    <!-- Modal Footer -->
    <template #footer>
      <div class="flex justify-between items-center w-full">
        <Button
          label="Revert to Defaults"
          size="small"
          severity="secondary"
          variant="text"
          class="text-xs"
          @click="revertToDefaults"
        >
          <template #icon>
            <RotateCcw class="w-3.5 h-3.5 mr-1" />
          </template>
        </Button>
        <Button label="Close" size="small" severity="secondary" @click="visible = false" />
      </div>
    </template>
  </Dialog>
</template>
