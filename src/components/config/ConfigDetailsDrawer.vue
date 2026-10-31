<script setup lang="ts">
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import { useResourceRawManifest } from '@/composables/useResourceRawManifest'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { KUBERNETES_POD_STATUS, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import type { ConfigMapInfo, SecretInfo, UsedByPod } from '@/types/kubernetes'
import { Eye, EyeOff, Shield, Tag as TagIcon } from '@lucide/vue'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import Tag from 'primevue/tag'
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

const props = defineProps<{
  visible: boolean
  resource: ConfigMapInfo | SecretInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const tableFilterStore = useTableFilterStore()

const activeTab = ref('overview')
const revealedKeys = ref<Record<string, boolean>>({})

// Reset states when resource changes
watch(
  () => props.resource,
  () => {
    revealedKeys.value = {}
    activeTab.value = 'overview'
  }
)

const isSecret = (res: ConfigMapInfo | SecretInfo): res is SecretInfo => {
  return 'type' in res
}

const toggleRevealKey = (key: string) => {
  revealedKeys.value[key] = !revealedKeys.value[key]
}

const decodeSecretValue = (val: string) => {
  try {
    return atob(val)
  } catch {
    return val // Return raw if not valid base64
  }
}

const handleNavigateToPod = (pod: UsedByPod) => {
  tableFilterStore.setFilter('pod', 'searchQuery', pod.name)
  if (pod.namespace) {
    tableFilterStore.setFilter('pod', 'selectedNamespace', [pod.namespace])
  }
  emit('update:visible', false)
  router.push({ path: '/workloads', query: { tab: 'pods' } })
}

const handleViewAllPods = () => {
  if (props.resource?.namespace) {
    tableFilterStore.setFilter('pod', 'selectedNamespace', [props.resource.namespace])
  }
  tableFilterStore.setFilter('pod', 'searchQuery', '')
  emit('update:visible', false)
  router.push({ path: '/workloads', query: { tab: 'pods' } })
}

const { displayedYaml, isYamlLoading, copied, copyYaml } = useResourceRawManifest({
  kind: computed(() =>
    props.resource
      ? isSecret(props.resource)
        ? KUBERNETES_RESOURCE_KIND.Secret
        : KUBERNETES_RESOURCE_KIND.ConfigMap
      : ''
  ),
  name: computed(() => props.resource?.name),
  namespace: computed(() => props.resource?.namespace),
  visible: computed(() => props.visible)
})
</script>

<template>
  <BaseResourceDrawer
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.resource"
    :title="props.resource?.name ?? ''"
    :kind="
      props.resource
        ? isSecret(props.resource)
          ? KUBERNETES_RESOURCE_KIND.Secret
          : KUBERNETES_RESOURCE_KIND.ConfigMap
        : ''
    "
    :kind-severity="props.resource ? (isSecret(props.resource) ? 'danger' : 'info') : 'info'"
    status-badge-class="bg-(--success)"
    :namespace="props.resource?.namespace"
    :age="props.resource?.age"
    @update:visible="emit('update:visible', $event)"
  >
    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="data" class="text-xs flex items-center gap-1.5 py-2.5 px-3">
        <Shield class="w-3.5 h-3.5" />
        <span>Data</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <div v-if="props.resource" class="flex flex-col gap-6">
        <!-- General Section -->
        <div>
          <h3 class="text-xs font-bold uppercase text-muted-color tracking-wider mb-3">General</h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-3 text-xs font-ui"
          >
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Namespace</div>
              <div class="col-span-2 text-xs text-primary font-mono">
                {{ props.resource.namespace }}
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Annotations</div>
              <div class="col-span-2 text-xs text-primary">
                {{ props.resource.annotations }}
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Created</div>
              <div class="col-span-2 text-xs text-primary">
                {{ props.resource.created }}
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Age</div>
              <div class="col-span-2 text-xs text-primary">
                <ReactiveAge :age="props.resource.age" />
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Resource Version</div>
              <div class="col-span-2 text-xs text-primary font-mono">
                {{ props.resource.resourceVersion }}
              </div>
            </div>
          </div>
        </div>

        <!-- Labels Section -->
        <KeyValueBadgeList :items="props.resource.labels" title="Labels" variant="tag" />

        <!-- Details Section -->
        <div>
          <h3 class="text-xs font-bold uppercase text-muted-color tracking-wider mb-3">Details</h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-3 text-xs font-ui"
          >
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Data Keys</div>
              <div class="col-span-2 text-xs text-primary">
                {{ props.resource.keysCount }}
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Size</div>
              <div class="col-span-2 text-xs text-primary font-mono">
                {{ props.resource.size }}
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Mounted In</div>
              <div class="col-span-2 text-xs text-primary">
                {{ props.resource.mountedPods }} pods
              </div>
            </div>
            <div class="grid grid-cols-3">
              <div class="text-xs text-muted-color font-medium">Immutable</div>
              <div class="col-span-2 text-xs text-primary">
                {{ props.resource.immutable ? 'True' : 'False' }}
              </div>
            </div>
          </div>
        </div>

        <!-- Used By Section -->
        <div>
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-xs font-bold uppercase text-muted-color tracking-wider">Used By</h3>
            <Button
              variant="text"
              size="small"
              class="text-xs text-primary hover:text-primary/80 p-0 font-semibold flex items-center gap-1 h-auto"
              :disabled="!props.resource?.mountedPods"
              @click="handleViewAllPods"
            >
              <span>View all ({{ props.resource?.mountedPods ?? 0 }})</span>
              <span class="text-[10px]">&rarr;</span>
            </Button>
          </div>
          <div class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-3 flex flex-col gap-2">
            <div
              v-for="pod in props.resource.usedBy"
              :key="pod.name"
              class="flex items-center justify-between p-2.5 rounded-[var(--ui-radius-card)] bg-(--bg-card) hover:bg-(--bg-hover) transition-colors cursor-pointer group"
              tabindex="0"
              role="button"
              @click="handleNavigateToPod(pod)"
              @keydown.enter.space.prevent="handleNavigateToPod(pod)"
            >
              <div class="flex items-center gap-2 min-w-0">
                <TagIcon
                  class="w-3.5 h-3.5 text-muted-color shrink-0 group-hover:text-primary transition-colors"
                />
                <div class="flex flex-col min-w-0">
                  <span class="text-xs font-mono text-primary truncate max-w-64 font-medium">
                    {{ pod.name }}
                  </span>
                  <span
                    v-if="pod.namespace && pod.namespace !== props.resource.namespace"
                    class="text-[10px] font-mono text-muted-color truncate"
                  >
                    {{ pod.namespace }}
                  </span>
                </div>
              </div>
              <div class="flex items-center gap-2 shrink-0">
                <Tag
                  v-for="mType in pod.mountType ? pod.mountType.split(', ') : []"
                  :key="mType"
                  rounded
                  severity="secondary"
                  :value="mType"
                  class="text-[10px] font-mono"
                />
                <Tag
                  rounded
                  :severity="pod.status === KUBERNETES_POD_STATUS.Running ? 'success' : 'secondary'"
                  :value="pod.status"
                />
              </div>
            </div>
            <div
              v-if="props.resource.usedBy.length === 0"
              class="p-4 text-center text-xs text-muted-color"
            >
              No pods currently referencing this configuration.
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Extra Panels -->
    <template #extra-panels>
      <TabPanel value="data">
        <div class="flex flex-col gap-4">
          <div
            v-for="(value, key) in props.resource?.data || {}"
            :key="key"
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-2"
          >
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold font-mono text-primary">{{ key }}</span>

              <Button
                v-if="props.resource && isSecret(props.resource)"
                v-tooltip.top="'Toggle visibility'"
                severity="secondary"
                variant="text"
                size="small"
                class="p-1 h-auto"
                @click="toggleRevealKey(key)"
              >
                <EyeOff v-if="revealedKeys[key]" class="w-3.5 h-3.5" />
                <Eye v-else class="w-3.5 h-3.5" />
              </Button>
            </div>

            <pre
              class="text-xs font-mono p-3 rounded-[var(--ui-radius-card)] bg-(--bg-card) overflow-x-auto whitespace-pre-wrap break-all select-all text-primary"
              >{{
                props.resource && isSecret(props.resource)
                  ? revealedKeys[key]
                    ? decodeSecretValue(value)
                    : '••••••••••••••••'
                  : value
              }}</pre>
          </div>
        </div>
      </TabPanel>
    </template>

    <!-- YAML Panel -->
    <template #yaml>
      <ResourceYamlTab
        :displayed-yaml="displayedYaml"
        :is-yaml-loading="isYamlLoading"
        :copied="copied"
        @copy-yaml="copyYaml"
      />
    </template>
  </BaseResourceDrawer>
</template>
