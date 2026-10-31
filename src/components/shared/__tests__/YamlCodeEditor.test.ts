import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('monacoEnvironment configures local Monaco worker without remote URLs', () => {
  const envPath = path.resolve('src/utils/monacoEnvironment.ts')
  assert.ok(fs.existsSync(envPath), 'src/utils/monacoEnvironment.ts must exist')
  const content = fs.readFileSync(envPath, 'utf-8')

  assert.ok(
    content.includes("from 'monaco-editor/editor/editor.worker?worker'"),
    'Must import local editor worker using Vite ?worker query'
  )
  assert.ok(
    !content.includes('http://') && !content.includes('https://'),
    'Must not include any external HTTP/HTTPS CDN URLs'
  )
  assert.ok(
    content.includes('globalScope.MonacoEnvironment'),
    'Must define MonacoEnvironment on global scope'
  )
})

test('YamlCodeEditor.vue satisfies offline architecture, lifecycle, and fallback requirements', () => {
  const editorPath = path.resolve('src/components/shared/YamlCodeEditor.vue')
  assert.ok(fs.existsSync(editorPath), 'src/components/shared/YamlCodeEditor.vue must exist')
  const content = fs.readFileSync(editorPath, 'utf-8')

  assert.ok(
    content.includes('data-component="yaml-code-editor"'),
    'Must have data-component="yaml-code-editor"'
  )
  assert.ok(
    content.includes("from 'monaco-editor/editor.js'"),
    "Must import 'monaco-editor/editor.js' directly"
  )
  assert.ok(
    content.includes("import 'monaco-editor/features/register.all.js'"),
    "Must register Monaco features via 'monaco-editor/features/register.all.js'"
  )
  assert.ok(
    content.includes("import 'monaco-editor/languages/definitions/yaml/register.js'"),
    "Must register YAML language definition via 'monaco-editor/languages/definitions/yaml/register.js'"
  )
  assert.ok(
    !content.includes('esm/vs/'),
    'Must use exported package subpaths without esm/vs/ prefix'
  )
  assert.ok(
    content.includes('onActivated(') && content.includes('layout()'),
    'Must call editorInstance.layout() onActivated for KeepAlive tab switches'
  )
  assert.ok(content.includes('dispose()'), 'Must dispose editor and model on unmount')
  assert.ok(
    content.includes('<Textarea'),
    'Must provide graceful PrimeVue Textarea fallback when initialization fails'
  )
})

test('YamlCodeEditorLoading.vue satisfies token hygiene and contract requirements', () => {
  const loadingPath = path.resolve('src/components/shared/YamlCodeEditorLoading.vue')
  assert.ok(
    fs.existsSync(loadingPath),
    'src/components/shared/YamlCodeEditorLoading.vue must exist'
  )
  const content = fs.readFileSync(loadingPath, 'utf-8')

  assert.ok(
    content.includes('data-component="yaml-code-editor-loading"'),
    'Must have data-component="yaml-code-editor-loading"'
  )
  assert.ok(
    content.includes('rounded-[var(--ui-radius-card)]'),
    'Must use var(--ui-radius-card) token'
  )
})

test('EditView lazy-loads YamlCodeEditor via defineAsyncComponent and uses controlled ToggleSwitch mode transitions', () => {
  const editViewPath = path.resolve('src/views/EditView.vue')
  const content = fs.readFileSync(editViewPath, 'utf-8')

  assert.ok(
    !content.includes('@guolao/vue-monaco-editor'),
    'Must not import legacy @guolao/vue-monaco-editor'
  )
  assert.ok(
    content.includes('defineAsyncComponent'),
    'Must use defineAsyncComponent for YamlCodeEditor'
  )
  assert.ok(
    content.includes('YamlCodeEditorLoading'),
    'Must configure YamlCodeEditorLoading as loadingComponent'
  )
  assert.ok(
    content.includes('transitionEditMode'),
    'Must use transitionEditMode for mode switching'
  )
  assert.ok(
    content.includes('applyMetadataFormValuesToResource'),
    'Must use applyMetadataFormValuesToResource for fallback metadata sync'
  )
  assert.ok(
    content.includes(':model-value="isYamlMode"'),
    'Must use controlled :model-value on ToggleSwitch'
  )
  assert.ok(
    !content.includes('v-model="isYamlMode"'),
    'Must not use uncontrolled v-model on ToggleSwitch'
  )
})

test('AppLayout forwards route params to keep-alive view and EditView guards against empty resource params', () => {
  const appLayoutPath = path.resolve('src/components/layout/AppLayout.vue')
  const appLayoutContent = fs.readFileSync(appLayoutPath, 'utf-8')

  assert.ok(
    appLayoutContent.includes('v-bind="currentRoute.params"'),
    'AppLayout must pass v-bind="currentRoute.params" to resolveViewComponent host'
  )

  const editViewPath = path.resolve('src/views/EditView.vue')
  const editViewContent = fs.readFileSync(editViewPath, 'utf-8')

  assert.ok(
    editViewContent.includes('resourceKind') && editViewContent.includes('resourceName'),
    'EditView must compute effective resource params with fallback'
  )
  assert.ok(
    editViewContent.includes('if (!kind || !name)'),
    'loadResource must guard against missing kind or name before calling getResourceRaw'
  )
})
