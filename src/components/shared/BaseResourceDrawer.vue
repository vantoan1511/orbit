<script setup lang="ts">
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import { Clock, FileCode, Server } from '@lucide/vue'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import Tag from 'primevue/tag'

const props = withDefaults(
  defineProps<{
    visible: boolean
    hasResource?: boolean
    title?: string
    kind?: string
    kindSeverity?: string
    statusBadgeClass?: string
    namespace?: string
    age?: string
    showYamlTab?: boolean
  }>(),
  {
    hasResource: true,
    title: '',
    kind: '',
    kindSeverity: 'info',
    statusBadgeClass: 'bg-emerald-500',
    namespace: undefined,
    age: undefined,
    showYamlTab: true
  }
)

const activeTab = defineModel<string>('activeTab', { default: 'overview' })

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()
</script>

<template>
  <Drawer
    :visible="props.visible"
    position="right"
    class="w-160! bg-(--bg-card)! border-l! border-(--border)!"
    :dismissable="true"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <div v-if="props.hasResource" class="flex items-center justify-between w-full pr-4">
        <!-- Resource Status, Title, Tags, Metadata -->
        <div class="flex items-center gap-3 min-w-0">
          <slot name="status">
            <span
              class="w-3 h-3 rounded-full shrink-0 animate-pulse"
              :class="props.statusBadgeClass"
            ></span>
          </slot>

          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <slot name="title">
                <h3
                  class="text-base font-bold text-primary font-mono truncate max-w-70"
                  :title="props.title"
                >
                  {{ props.title }}
                </h3>
              </slot>

              <slot name="tags">
                <Tag
                  v-if="props.kind"
                  rounded
                  class="font-mono"
                  :severity="props.kindSeverity"
                  :value="props.kind"
                />
              </slot>
            </div>

            <slot name="metadata">
              <div
                v-if="props.namespace || props.age"
                class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5"
              >
                <span v-if="props.namespace">ns: {{ props.namespace }}</span>
                <span v-if="props.namespace && props.age" class="text-muted-color/60">•</span>
                <span v-if="props.age" class="flex items-center gap-1">
                  <Clock class="w-3 h-3" />
                  <ReactiveAge :age="props.age" />
                </span>
              </div>
            </slot>
          </div>
        </div>

        <!-- Quick Actions Slot -->
        <div v-if="$slots.actions" class="flex items-center gap-2 shrink-0">
          <slot name="actions"></slot>
        </div>
      </div>
    </template>

    <!-- Main Content Body -->
    <div v-if="props.hasResource" class="flex flex-col h-full">
      <Tabs v-model:value="activeTab" class="flex flex-col flex-1 min-h-0">
        <TabList class="bg-transparent! border-b! border-(--border)! px-2">
          <Tab value="overview" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Server class="w-3.5 h-3.5" />
            <span>Overview</span>
          </Tab>

          <slot name="extra-tabs"></slot>

          <Tab
            v-if="props.showYamlTab"
            value="yaml"
            class="text-xs! flex items-center gap-1.5 py-2.5 px-3"
          >
            <FileCode class="w-3.5 h-3.5" />
            <span>YAML</span>
          </Tab>
        </TabList>

        <TabPanels class="flex-1 overflow-y-auto p-6! bg-transparent!">
          <TabPanel value="overview">
            <slot name="overview">
              <slot></slot>
            </slot>
          </TabPanel>

          <slot name="extra-panels"></slot>

          <TabPanel v-if="props.showYamlTab" value="yaml" class="h-full">
            <slot name="yaml"></slot>
          </TabPanel>
        </TabPanels>
      </Tabs>
    </div>
  </Drawer>
</template>
