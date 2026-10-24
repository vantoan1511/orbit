# Recipe: Canonical Kubernetes Resource Page

Follow this recipe when adding or refactoring a Kubernetes resource page (e.g., StatefulSets, Ingresses, ConfigMaps, Secrets, Jobs).

---

## Architecture Flow

```
Rust Engine                     Neutralino IPC                  Vue 3 Pinia & View
[Watcher / Fetcher]   ──►   [Event: ResourceUpdated]  ──►   [kubernetesStore]
                                                                   │
                                                                   ▼
                                                            [ResourceView.vue]
                                                            (PrimeVue DataTable)
                                                                   │
                                                                   ▼
                                                            [ResourceDrawer.vue]
```

---

## 1. View Component Anatomy (`src/views/[Resource]View.vue`)

Use the established Orbit layout: flat, borderless, compact toolbar with namespace filter, search, and a flat `DataTable`.

```vue
<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-(--bg-app)">
    <!-- Header Toolbar -->
    <div class="flex items-center justify-between px-6 py-3 border-b border-(--border) bg-(--bg-card)">
      <div class="flex items-center gap-3">
        <h1 class="text-xl font-bold tracking-tight text-(--text-primary)">[Resource Plural]</h1>
        <Tag :value="filteredItems.length" severity="secondary" />
      </div>

      <div class="flex items-center gap-2">
        <!-- Namespace filter & Search (PrimeVue controls) -->
        <InputText
          v-model="searchQuery"
          placeholder="Filter resources..."
          size="small"
          class="w-64"
        />
        <Button
          icon="pi pi-refresh"
          severity="secondary"
          variant="outlined"
          size="small"
          v-tooltip.top="'Refresh'"
          @click="refresh"
        />
      </div>
    </div>

    <!-- Data Table -->
    <div class="flex-1 overflow-auto p-4">
      <DataTable
        :value="filteredItems"
        data-key="metadata.uid"
        size="small"
        :paginator="true"
        :rows="25"
        class="border border-(--border) rounded-lg overflow-hidden"
        @row-click="onRowClick"
      >
        <Column field="metadata.name" header="Name" sortable>
          <template #body="{ data }">
            <span class="font-mono text-xs font-medium text-(--accent)">
              {{ data.metadata.name }}
            </span>
          </template>
        </Column>

        <Column field="metadata.namespace" header="Namespace" sortable />

        <Column field="status" header="Status">
          <template #body="{ data }">
            <!-- Reuse StatusBadge / semantic styling -->
          </template>
        </Column>

        <Column field="metadata.creationTimestamp" header="Age" sortable />

        <Column class="w-12">
          <template #body="{ data }">
            <!-- Context Menu / Actions Button -->
            <Button
              icon="pi pi-ellipsis-v"
              severity="secondary"
              variant="text"
              size="small"
              v-tooltip.left="'Actions'"
              @click.stop="openActions(data)"
            />
          </template>
        </Column>
      </DataTable>
    </div>

    <!-- Details Drawer -->
    <[Resource]DetailsDrawer
      v-if="selectedItem"
      :visible="isDrawerOpen"
      :resource="selectedItem"
      @close="isDrawerOpen = false"
    />
  </div>
</template>
```

---

## 2. Details Drawer Anatomy (`src/components/[domain]/[Resource]DetailsDrawer.vue`)

Use `BaseResourceDrawer` as the container wrapper:

```vue
<template>
  <BaseResourceDrawer
    :visible="visible"
    :title="resource.metadata.name"
    :subtitle="resource.metadata.namespace"
    @close="$emit('close')"
  >
    <template #actions>
      <Button
        severity="secondary"
        size="small"
        variant="outlined"
        v-tooltip.top="'View Logs'"
        @click="viewLogs"
      >
        <Terminal class="w-3.5 h-3.5 mr-1" />
        <span>Logs</span>
      </Button>
      <Button
        severity="secondary"
        size="small"
        variant="outlined"
        v-tooltip.top="'Edit YAML'"
        @click="editYaml"
      >
        <FileCode class="w-3.5 h-3.5 mr-1" />
        <span>YAML</span>
      </Button>
    </template>

    <!-- Tab Sections: Overview, Events, YAML -->
    <Tabs value="overview">
      <TabList>
        <Tab value="overview">Overview</Tab>
        <Tab value="events">Events</Tab>
        <Tab value="yaml">YAML</Tab>
      </TabList>
      <TabPanels>
        <TabPanel value="overview">
          <!-- Overview metrics & KeyValue lists -->
        </TabPanel>
      </TabPanels>
    </Tabs>
  </BaseResourceDrawer>
</template>
```

---

## Non-Negotiable Checklist for New Resource Pages
- [ ] No raw `<button>`, `<input>`, `<select>` (only PrimeVue components).
- [ ] No native `title="..."` attributes (use `v-tooltip` directive).
- [ ] No hardcoded hex color codes in style blocks (use CSS tokens from `src/assets/base.css`).
- [ ] Numeric and metrics columns in tables are right-aligned.
- [ ] Unit tests for data transformation/sorting written in `src/**/__tests__/*.test.ts`.
