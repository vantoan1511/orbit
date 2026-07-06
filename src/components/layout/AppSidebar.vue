<script setup lang="ts">
import { ref } from "vue";
import { useRoute } from "vue-router";
import {
  LayoutDashboard,
  Server,
  Boxes,
  Box,
  Network,
  Settings2,
  HardDrive,
  FolderOpen,
  Activity,
  ShieldCheck,
  Settings,
  Plus,
  Sun,
  Moon,
  BookOpen,
  Bell,
  User
} from "@lucide/vue";

// Define clusters list
const clusters = ref([
  { id: "prod-us-east", name: "production-us-east-1", status: "healthy" },
  { id: "staging-eu-west", name: "staging-eu-west-1", status: "offline" },
  { id: "local-dev", name: "local-dev", status: "offline" }
]);

const activeCluster = ref("prod-us-east");

// Navigation links
const navLinks = [
  { name: "Overview", icon: LayoutDashboard, path: "/" },
  { name: "Nodes", icon: Server, path: "/nodes" },
  { name: "Workloads", icon: Boxes, path: "/workloads" },
  { name: "Pods", icon: Box, path: "/pods" },
  { name: "Services", icon: Network, path: "/services" },
  { name: "ConfigMaps & Secrets", icon: Settings2, path: "/config" },
  { name: "Storage", icon: HardDrive, path: "/storage" },
  { name: "Namespaces", icon: FolderOpen, path: "/namespaces" },
  { name: "Events", icon: Activity, path: "/events" },
  { name: "Policies", icon: ShieldCheck, path: "/policies" },
  { name: "Settings", icon: Settings, path: "/settings" }
];

const route = useRoute();

// Theme state
const isDark = ref(true);

const toggleTheme = () => {
  isDark.value = !isDark.value;
  const html = document.documentElement;
  if (isDark.value) {
    html.classList.add("my-app-dark");
    html.setAttribute("data-theme", "dark");
  } else {
    html.classList.remove("my-app-dark");
    html.setAttribute("data-theme", "light");
  }
};
</script>

<template>
  <aside class="w-64 bg-[var(--bg-sidebar)] border-r border-[var(--border)] flex flex-col h-screen text-[var(--text-primary)] select-none">
    <!-- Brand Header -->
    <div class="h-16 px-6 flex items-center gap-3 border-b border-[var(--border)]">
      <!-- Orbit Icon Logo -->
      <div class="w-6 h-6 rounded-full border-2 border-[var(--accent)] flex items-center justify-center relative">
        <div class="w-2.5 h-2.5 rounded-full bg-[var(--accent)]"></div>
        <div class="absolute -right-0.5 -bottom-0.5 w-1.5 h-1.5 rounded-full bg-emerald-500 border border-[var(--bg-sidebar)]"></div>
      </div>
      <span class="text-xl font-bold tracking-tight font-ui">Orbit</span>
    </div>

    <!-- Clusters Section -->
    <div class="p-4 border-b border-[var(--border)]">
      <div class="text-[10px] font-bold text-[var(--text-muted)] tracking-wider uppercase mb-2 px-2">
        Clusters
      </div>
      <div class="flex flex-col gap-1">
        <button
          v-for="cluster in clusters"
          :key="cluster.id"
          @click="activeCluster = cluster.id"
          class="w-full text-left px-3 py-2 rounded-lg text-sm flex items-center justify-between transition-all duration-200"
          :class="[
            activeCluster === cluster.id
              ? 'bg-[var(--bg-active)] font-medium text-[var(--text-primary)]'
              : 'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]'
          ]"
        >
          <div class="flex items-center gap-2 truncate">
            <span 
              class="w-2 h-2 rounded-full flex-shrink-0"
              :class="cluster.status === 'healthy' ? 'bg-emerald-500' : 'bg-zinc-500'"
            ></span>
            <span class="truncate">{{ cluster.name }}</span>
          </div>
        </button>

        <button class="w-full text-left px-3 py-2 rounded-lg text-sm text-[var(--text-muted)] hover:bg-[var(--bg-hover)] flex items-center gap-2 transition-all duration-200">
          <Plus class="w-4 h-4" />
          <span>Add Cluster</span>
        </button>
      </div>
    </div>

    <!-- Navigation Section -->
    <nav class="flex-1 overflow-y-auto p-4 space-y-1">
      <router-link
        v-for="link in navLinks"
        :key="link.name"
        :to="link.path"
        class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-all duration-200"
        :class="[
          route.path === link.path
            ? 'bg-[var(--bg-active)] font-medium text-[var(--text-primary)]'
            : 'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]'
        ]"
      >
        <component :is="link.icon" class="w-4 h-4 flex-shrink-0" />
        <span>{{ link.name }}</span>
      </router-link>
    </nav>

    <!-- Bottom Footer -->
    <div class="p-4 border-t border-[var(--border)] flex items-center justify-between bg-[var(--bg-sidebar)]">
      <div class="flex items-center gap-3">
        <!-- Theme Toggle -->
        <button 
          @click="toggleTheme" 
          class="p-2 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] transition-all duration-200"
          title="Toggle Theme"
        >
          <Sun v-if="isDark" class="w-4.5 h-4.5" />
          <Moon v-else class="w-4.5 h-4.5" />
        </button>

        <!-- Docs -->
        <a 
          href="#" 
          class="p-2 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] transition-all duration-200"
          title="Documentation"
        >
          <BookOpen class="w-4.5 h-4.5" />
        </a>

        <!-- Notifications -->
        <button 
          class="p-2 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] transition-all duration-200 relative"
          title="Notifications"
        >
          <Bell class="w-4.5 h-4.5" />
          <span class="absolute top-1 right-1 w-2 h-2 rounded-full bg-rose-500"></span>
        </button>
      </div>

      <!-- Profile & Version -->
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-[var(--text-muted)] font-mono">v0.1.0</span>
        <button class="w-8 h-8 rounded-full bg-[var(--bg-hover)] flex items-center justify-center text-[var(--text-secondary)] hover:bg-[var(--bg-active)] transition-all duration-200">
          <User class="w-4 h-4" />
        </button>
      </div>
    </div>
  </aside>
</template>
