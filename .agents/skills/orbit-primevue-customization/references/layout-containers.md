# Layout & Container Patterns

This reference provides implementation patterns for organizing content and providing feedback using PrimeVue 4 components.

## 🧱 Structural Containers

### 1. Cards & Advanced Layouts
**Tabs & Stepper (PV4 Syntax)**:
```vue
<!-- Tabs -->
<Tabs value="0">
  <TabList>
    <Tab value="0">Account</Tab>
    <Tab value="1">Security</Tab>
  </TabList>
  <TabPanels>
    <TabPanel value="0">Content...</TabPanel>
  </TabPanels>
</Tabs>

<!-- Stepper -->
<Stepper value="1">
  <StepList>
    <Step value="1">Step 1</Step>
    <Step value="2">Step 2</Step>
  </StepList>
  <StepPanels>
    <StepPanel v-slot="{ activateCallback }" value="1">
      <Button label="Next" @click="activateCallback('2')" />
    </StepPanel>
  </StepPanels>
</Stepper>
```

---

## ⚓ Navigation Systems

**Menubar (Navbar)**:
```vue
<Menubar :model="items">
  <template #start>
    <img src="/logo.svg" class="h-8" />
  </template>
  <template #item="{ item, props, hasSubmenu, root }">
    <router-link v-if="item.route" :to="item.route" v-slot="{ href, navigate }" custom>
      <a :href="href" v-bind="props.action" @click="navigate">
        <i :class="item.icon" />
        <span>{{ item.label }}</span>
      </a>
    </router-link>
  </template>
</Menubar>
```

**PanelMenu (Activity Bar + Contextual Sidebar)**:
When using `PanelMenu` in a contextual sidebar panel alongside an activity bar, filter `:model` to render only the category group corresponding to the selected activity bar tab:
```vue
<PanelMenu
  :model="currentCategoryPanelItems"
  v-model:expandedKeys="expandedKeys"
  multiple
>
  <template #item="{ item, active }">
    <!-- Category Header Group -->
    <div v-if="item.items && item.items.length > 0">...</div>
    <!-- Leaf Navigation Link -->
    <router-link v-else-if="item.route" :to="item.route">...</router-link>
  </template>
</PanelMenu>
```

---

## 🏷️ Status & Informational

- **`Avatar`**: Represents users via labels, icons, or images. Use `AvatarGroup` for stacks.
- **`Badge`**: Small status indicators. Use `OverlayBadge` to attach to other elements.
- **`Chip` / `Tag`**: Categorization or status labels. Chips are often removable.
- **`Skeleton` / `ProgressBar` / `ProgressSpinner`**: Indicators for loading or background tasks.
- **`Message` / `Toast`**: Inline or overlay feedback for system events.
