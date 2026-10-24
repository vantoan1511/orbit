# Input Components & Form Patterns

This reference provides implementation patterns for PrimeVue 4 input components and form management within Nuxt 4.

## 🚀 Core Concepts (PrimeVue 4)

| Prop | Purpose | Values |
| :--- | :--- | :--- |
| `variant` | Defines the visual style of the input. | `"outlined"` (default), `"filled"` |
| `fluid` | Makes the component span 100% of its container. | `boolean` (Default: `false`) |
| `size` | Adjusts the component's scale. | `"small"`, `"large"` |
| `invalid` | Visually indicates an error state. | `boolean` |

### Global Fluid Layout
Instead of applying `fluid` to every input, wrap them in a `<Fluid>` container:
```vue
<Fluid>
  <div class="grid grid-cols-2 gap-4">
    <InputText placeholder="First Name" />
    <InputText placeholder="Last Name" />
  </div>
</Fluid>
```

---

## 📝 Input Components

### 1. Selection & Hierarchical Data
**Select (Dropdown)**: Standard and grouped selection.
```vue
<Select 
  v-model="selected" 
  :options="groupedItems" 
  optionLabel="label" 
  optionGroupLabel="group" 
  optionGroupChildren="items"
  filter 
  showClear 
  fluid 
/>
```

**MultiSelect & TreeSelect**:
```vue
<!-- MultiSelect with Tags -->
<MultiSelect v-model="selected" :options="opts" display="chip" fluid />

<!-- Hierarchical Selection -->
<TreeSelect v-model="node" :options="nodes" selectionMode="checkbox" display="chip" fluid />
```

### 2. Specialized Inputs
**DatePicker**: Support for ranges and time.
```vue
<DatePicker v-model="dates" selectionMode="range" showTime hourFormat="24" fluid />
```

**Checkbox & Radio Groups**:
```vue
<CheckboxGroup v-model="values" class="flex gap-4">
  <div v-for="opt in options" :key="opt.key" class="flex items-center gap-2">
    <Checkbox :inputId="opt.key" :value="opt.value" />
    <label :for="opt.key">{{ opt.label }}</label>
  </div>
</CheckboxGroup>
```

### 3. Buttons & Multi-Action
**SplitButton & SpeedDial**:
```vue
<!-- Default action + dropdown -->
<SplitButton label="Save" icon="pi pi-save" :model="items" @click="save" severity="success" />

<!-- Floating actions -->
<SpeedDial :model="actions" direction="up" :style="{ position: 'absolute', bottom: '1rem', right: '1rem' }" />
```

---

## 🛠️ Form Management Patterns

### Validation States
Apply the `invalid` prop based on your validation logic (e.g., VeeValidate, Zod).
```vue
<InputText :invalid="v$.email.$error" v-model="email" />
<Message v-if="v$.email.$error" severity="error" size="small">Invalid email</Message>
```

### Performance (Large Lists)
Enable virtual scrolling for components like `Select` or `MultiSelect` when dealing with 1000+ items:
```vue
<Select :options="largeData" :virtualScrollerOptions="{ itemSize: 38 }" />
```
```
