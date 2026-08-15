<script setup lang="ts">
import StringListEditor from '@/components/shared/StringListEditor.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { isValidHost, isValidK8sName, parseRuleSummary } from '@/utils/validators'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, inject, onMounted, ref, type Ref } from 'vue'

interface CloneDialogData {
  sourceName: string
  sourceNamespace: string
  sourceHosts: string[]
}

interface CloneDialogResult {
  newName: string
  newNamespace: string
  newHosts: string[]
}

const dialogRef = inject<
  | Ref<{
      data: CloneDialogData
      close: (data?: CloneDialogResult) => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()
const namespaceOptions = computed(() => k8sStore.namespaceList.map((ns) => ns.name))

const newName = ref('')
const newNamespace = ref('')
const sourceName = ref('')
const hosts = ref<string[]>([])

onMounted(() => {
  if (dialogRef?.value?.data) {
    sourceName.value = dialogRef.value.data.sourceName
    newName.value = `${dialogRef.value.data.sourceName}-copy`
    newNamespace.value = dialogRef.value.data.sourceNamespace
    hosts.value = [...dialogRef.value.data.sourceHosts]
  }
})

const sourceIngress = computed(() => {
  return k8sStore.ingresses.find(
    (ing) =>
      ing.name === sourceName.value &&
      ing.namespace === (dialogRef?.value?.data?.sourceNamespace || 'default')
  )
})

const sourceRules = computed(() => {
  if (!sourceIngress.value?.rulesSummary) return []
  return sourceIngress.value.rulesSummary
    .map(parseRuleSummary)
    .filter((r): r is { host: string; path: string } => r !== null)
})

const existingIngressRulesMap = computed(() => {
  const map = new Map<string, { ingressName: string; namespace: string }>()
  for (const ing of k8sStore.ingresses) {
    if (ing.namespace === newNamespace.value) {
      if (ing.rulesSummary) {
        for (const ruleStr of ing.rulesSummary) {
          const parsed = parseRuleSummary(ruleStr)
          if (parsed) {
            const key = `${parsed.host}:::${parsed.path}`
            if (!map.has(key)) {
              map.set(key, { ingressName: ing.name, namespace: ing.namespace })
            }
          }
        }
      }
    }
  }
  return map
})

const nameErrorMessage = computed(() => {
  const trimmed = newName.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Name must be a valid DNS-1123 subdomain (lowercase letters, numbers, hyphens, dots).'
  }
  const exists = k8sStore.ingresses.some(
    (ing) =>
      ing.namespace === newNamespace.value && ing.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `An Ingress named "${trimmed}" already exists in namespace "${newNamespace.value}".`
  }
  return null
})

const validateHostItem = (host: string, index: number, allHosts: string[]): boolean | string => {
  const trimmed = host.trim()
  if (!trimmed) return true
  if (!isValidHost(trimmed)) {
    return 'Must be a valid hostname (e.g. example.com or *.example.com).'
  }
  const lower = trimmed.toLowerCase()
  const sourcePath = sourceRules.value[index]?.path || '/'

  // Check duplicate in current dialog list with same path
  const dupIndex = allHosts.findIndex(
    (h, idx) =>
      idx !== index &&
      h.trim().toLowerCase() === lower &&
      (sourceRules.value[idx]?.path || '/') === sourcePath
  )
  if (dupIndex !== -1) {
    return `Duplicate host "${trimmed}" with path "${sourcePath}" in list.`
  }

  // Check if (host + path) is already in use by an existing Ingress in target namespace
  const key = `${lower}:::${sourcePath}`
  const existing = existingIngressRulesMap.value.get(key)
  if (existing) {
    return `Host "${trimmed}" with path "${sourcePath}" is already used by Ingress "${existing.ingressName}".`
  }
  return true
}

const areHostsValid = computed(() => {
  return hosts.value.every((h, idx) => validateHostItem(h, idx, hosts.value) === true)
})

const isValid = computed(
  () =>
    Boolean(newName.value.trim()) &&
    !nameErrorMessage.value &&
    Boolean(newNamespace.value) &&
    areHostsValid.value
)

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleClone = () => {
  if (!isValid.value) return
  dialogRef?.value?.close({
    newName: newName.value.trim(),
    newNamespace: newNamespace.value,
    newHosts: hosts.value.map((h) => h.trim()).filter(Boolean)
  })
}
</script>

<template>
  <form @submit.prevent="handleClone" class="flex flex-col gap-3.5">
    <p class="text-xs text-muted-color">
      Cloning
      <span class="font-semibold text-primary">{{ sourceName }}</span>
      into a new Ingress:
    </p>

    <div class="flex flex-col gap-1.5">
      <label for="clone-ingress-name" class="text-xs font-semibold text-muted-color"
        >New Name</label
      >
      <InputText
        id="clone-ingress-name"
        v-model="newName"
        fluid
        size="small"
        :invalid="Boolean(newName.trim() && nameErrorMessage)"
        class="text-xs"
      />
      <small
        v-if="newName.trim() && nameErrorMessage"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nameErrorMessage }}
      </small>
    </div>

    <div class="flex flex-col gap-1.5">
      <label for="clone-ingress-namespace" class="text-xs font-semibold text-muted-color">
        Namespace
      </label>
      <Select
        id="clone-ingress-namespace"
        v-model="newNamespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <StringListEditor
      v-model="hosts"
      title="Hosts"
      add-label="Add Host"
      placeholder="e.g. example.com"
      :item-validator="validateHostItem"
    />

    <div class="flex justify-end gap-2 pt-2">
      <Button
        type="button"
        label="Cancel"
        severity="secondary"
        variant="text"
        size="small"
        @click="handleCancel"
      />
      <Button type="submit" label="Clone" severity="primary" size="small" :disabled="!isValid" />
    </div>
  </form>
</template>
