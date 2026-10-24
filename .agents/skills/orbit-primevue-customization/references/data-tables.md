# Data Display & Visualization

This reference provides implementation patterns for displaying and visualizing data using PrimeVue 4 components.

## 📊 Tabular & Grid Data

### 1. DataTable (Lazy Load & Templates)
```vue
<DataTable 
  lazy paginator 
  v-model:filters="filters"
  :value="items" :totalRecords="total" :loading="loading"
  @page="onPage" @sort="onSort" @filter="onFilter"
>
  <Column field="name" header="Name" sortable>
    <template #body="{ data }">
      <Skeleton v-if="loading" />
      <span v-else>{{ data.name }}</span>
    </template>
  </Column>
  <Column header="Status">
    <template #body="{ data }">
      <Tag :value="data.status" :severity="getSeverity(data)" />
    </template>
  </Column>
</DataTable>
```

### 2. DataView (Grid Pattern)
```vue
<DataView :value="products" layout="grid">
  <template #grid="{ items }">
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div v-for="item in items" :key="item.id" class="p-4 border rounded-lg bg-surface-0 dark:bg-surface-900">
        <img :src="item.image" :alt="item.name" class="w-full rounded mb-4" />
        <div class="font-bold">{{ item.name }}</div>
      </div>
    </div>
  </template>
</DataView>
```

---

## 🌳 Hierarchical Data

**TreeTable**: Exactly one column must have the `expander` prop.
```vue
<TreeTable :value="nodes" v-model:expandedKeys="expandedKeys">
  <Column field="name" header="Name" expander />
  <Column field="size" header="Size" />
</TreeTable>
```

---

## 🖼️ Media & Collections

- **`Carousel`**: Content slider with `numVisible` and `autoplayInterval`.
- **`Galleria`**: Advanced image gallery with thumbnails and fullscreen mode.
- **`Image`**: Single image with `preview` (zoom/rotate) and `alt` text.
- **`ImageCompare`**: Side-by-side comparison with a slider.
- **`OrderList` / `PickList`**: Manage item ordering or move items between collections.

---

## 📈 Visualization & Performance

### 1. Charts & Meters
- **`Chart`**: Based on Chart.js. Use design tokens (e.g., `$dt('primary.color')`) for colors.
- **`MeterGroup`**: Displays scalar measurements (e.g., storage usage) with optional icons.

### 2. Timelines & Performance
- **`Timeline`**: Process milestones. Supports `align="alternate"`.
- **`VirtualScroller`**: Renders 10,000+ items efficiently using fixed `itemSize`.
- **`Paginator`**: Standalone pagination control for custom data flows.
