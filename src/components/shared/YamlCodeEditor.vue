<script setup lang="ts">
import { AlertTriangle } from '@lucide/vue'
import * as monaco from 'monaco-editor/editor.js'
import 'monaco-editor/features/register.all.js'
import 'monaco-editor/languages/definitions/yaml/register.js'
import Textarea from 'primevue/textarea'
import { onActivated, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ensureMonacoEnvironment } from '@/utils/monacoEnvironment'

interface Props {
  modelValue: string
  isDark?: boolean
  readOnly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  isDark: false,
  readOnly: false
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const containerRef = ref<HTMLDivElement | null>(null)
const hasInitFailed = ref(false)

let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null
let model: monaco.editor.ITextModel | null = null
let contentSubscription: monaco.IDisposable | null = null
let isApplyingExternalValue = false

onMounted(() => {
  try {
    ensureMonacoEnvironment()
    if (containerRef.value) {
      model = monaco.editor.createModel(props.modelValue || '', 'yaml')
      editorInstance = monaco.editor.create(containerRef.value, {
        model,
        theme: props.isDark ? 'vs-dark' : 'vs',
        automaticLayout: true,
        minimap: { enabled: true },
        fontSize: 13,
        lineHeight: 22,
        scrollBeyondLastLine: false,
        roundedSelection: false,
        padding: { top: 16, bottom: 16 },
        readOnly: Boolean(props.readOnly),
        tabSize: 2,
        insertSpaces: true
      })

      contentSubscription = editorInstance.onDidChangeModelContent(() => {
        if (isApplyingExternalValue) return
        emit('update:modelValue', editorInstance?.getValue() ?? '')
      })
    }
  } catch (err) {
    console.error('Failed to initialize Monaco YAML editor:', err)
    hasInitFailed.value = true
  }
})

watch(
  () => props.modelValue,
  (next) => {
    const nextVal = next ?? ''
    if (editorInstance && editorInstance.getValue() !== nextVal) {
      isApplyingExternalValue = true
      editorInstance.setValue(nextVal)
      isApplyingExternalValue = false
    }
  }
)

watch(
  () => props.isDark,
  (dark) => {
    monaco.editor.setTheme(dark ? 'vs-dark' : 'vs')
  }
)

watch(
  () => props.readOnly,
  (ro) => {
    editorInstance?.updateOptions({ readOnly: Boolean(ro) })
  }
)

onActivated(() => {
  editorInstance?.layout()
})

onBeforeUnmount(() => {
  contentSubscription?.dispose()
  editorInstance?.dispose()
  model?.dispose()
})
</script>

<template>
  <div
    data-component="yaml-code-editor"
    class="w-full h-full flex flex-col rounded-[var(--ui-radius-card)] border border-(--border) bg-(--bg-card) overflow-hidden"
  >
    <div v-if="!hasInitFailed" ref="containerRef" class="w-full h-full flex-1 min-h-0" />
    <template v-else>
      <div class="px-3 py-2 bg-(--bg-hover)/40 flex items-center gap-2 text-xs text-muted-color">
        <AlertTriangle class="w-3.5 h-3.5 text-(--warning) shrink-0" />
        <span>Editor failed to load. Fallback text mode is active.</span>
      </div>
      <Textarea
        :model-value="props.modelValue"
        :readonly="props.readOnly"
        fluid
        class="flex-1 w-full h-full resize-none border-0 font-mono text-xs leading-relaxed p-4 bg-transparent"
        @update:model-value="emit('update:modelValue', String($event ?? ''))"
      />
    </template>
  </div>
</template>
