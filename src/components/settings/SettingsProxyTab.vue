<script setup lang="ts">
import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settingsStore'
import { CONFIG_KEYS, type ProxySettings } from '@/types/settings'
import { RotateCcw } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'

const settingsStore = useSettingsStore()
const toast = useToast()

const DEFAULT_PROXY_SETTINGS: ProxySettings = {
  enabled: false,
  httpProxy: '',
  httpsProxy: '',
  noProxy: ''
}

const proxySettings = computed<ProxySettings>(() => {
  return settingsStore.getConfigValue<ProxySettings>(
    CONFIG_KEYS.PROXY_SETTINGS,
    DEFAULT_PROXY_SETTINGS
  )
})

const isModified = computed(() => {
  const current = proxySettings.value
  return (
    current.enabled !== DEFAULT_PROXY_SETTINGS.enabled ||
    current.httpProxy !== DEFAULT_PROXY_SETTINGS.httpProxy ||
    current.httpsProxy !== DEFAULT_PROXY_SETTINGS.httpsProxy ||
    current.noProxy !== DEFAULT_PROXY_SETTINGS.noProxy
  )
})

async function updateProxyField<K extends keyof ProxySettings>(field: K, val: ProxySettings[K]) {
  const updated: ProxySettings = {
    ...proxySettings.value,
    [field]: val
  }
  await settingsStore.updateConfigValue(CONFIG_KEYS.PROXY_SETTINGS, updated)
}

async function handleReset() {
  await settingsStore.updateConfigValue(CONFIG_KEYS.PROXY_SETTINGS, DEFAULT_PROXY_SETTINGS)
  toast.add({
    severity: 'success',
    summary: 'Proxy Settings Reset',
    detail: 'Proxy configuration has been reset to defaults.',
    life: 2500
  })
}
</script>

<template>
  <div data-component="settings-proxy-tab" class="flex flex-col gap-10">
    <!-- Header row with Title and Reset Action -->
    <div class="flex items-start justify-between gap-4">
      <div>
        <h3 class="text-base font-semibold text-primary">HTTP &amp; HTTPS Proxy</h3>
        <p class="text-xs text-muted-color mt-1">
          Configure network proxy routing for outbound cluster connections and Kubernetes APIs.
        </p>
      </div>

      <Button
        type="button"
        :disabled="!isModified"
        v-tooltip.top="
          isModified
            ? 'Reset proxy configuration to defaults'
            : 'Proxy settings are currently set to defaults'
        "
        severity="secondary"
        variant="text"
        size="small"
        class="text-xs text-muted-color hover:text-primary shrink-0 transition-opacity flex items-center gap-1.5"
        @click="handleReset"
      >
        <RotateCcw class="w-3.5 h-3.5" />
        <span>Reset to Defaults</span>
      </Button>
    </div>

    <!-- Section 1: Master Proxy Switch -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Proxy Status</h4>
        <p class="text-[11px] text-muted-color">
          Enable or disable outbound HTTP/HTTPS proxy routing globally across all cluster calls.
        </p>
      </div>

      <div
        class="w-full md:w-2/3 xl:w-3/4 flex items-center justify-between p-4 bg-surface-50 dark:bg-surface-900 rounded-[var(--ui-radius-card)]"
      >
        <div class="flex flex-col gap-0.5">
          <span class="text-xs font-medium text-primary">Enable Network Proxy</span>
          <span class="text-[11px] text-muted-color">
            Route API calls through configured proxy endpoints instead of direct connections
          </span>
        </div>
        <ToggleSwitch
          :model-value="proxySettings.enabled"
          @update:model-value="(val) => updateProxyField('enabled', Boolean(val))"
        />
      </div>
    </div>

    <!-- Section 2: HTTP Proxy -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">HTTP Proxy</h4>
        <p class="text-[11px] text-muted-color">
          Proxy server URL for standard unencrypted HTTP traffic.
        </p>
      </div>

      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-1.5">
        <InputText
          :disabled="!proxySettings.enabled"
          :model-value="proxySettings.httpProxy"
          placeholder="http://proxy.corp.internal:8080"
          class="w-full text-xs font-mono"
          @update:model-value="(val) => updateProxyField('httpProxy', String(val ?? ''))"
        />
        <span class="text-[11px] text-muted-color">
          Format: http://[user:password@]host[:port]
        </span>
      </div>
    </div>

    <!-- Section 3: HTTPS Proxy -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">HTTPS Proxy</h4>
        <p class="text-[11px] text-muted-color">
          Proxy server URL for secure TLS/HTTPS Kubernetes API traffic.
        </p>
      </div>

      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-1.5">
        <InputText
          :disabled="!proxySettings.enabled"
          :model-value="proxySettings.httpsProxy"
          placeholder="http://proxy.corp.internal:8443"
          class="w-full text-xs font-mono"
          @update:model-value="(val) => updateProxyField('httpsProxy', String(val ?? ''))"
        />
        <span class="text-[11px] text-muted-color">
          Used by default for all secure apiserver connections if defined
        </span>
      </div>
    </div>

    <!-- Section 4: Bypass List (noProxy) -->
    <div class="flex flex-col md:flex-row gap-6 lg:gap-10">
      <div class="w-full md:w-1/3 xl:w-1/4 flex flex-col gap-1 shrink-0">
        <h4 class="text-xs font-semibold text-primary uppercase tracking-wider">Bypass List</h4>
        <p class="text-[11px] text-muted-color">
          Endpoints that should be contacted directly without going through any proxy.
        </p>
      </div>

      <div class="w-full md:w-2/3 xl:w-3/4 flex flex-col gap-1.5">
        <InputText
          :disabled="!proxySettings.enabled"
          :model-value="proxySettings.noProxy"
          placeholder="localhost,127.0.0.1,.cluster.local,.internal"
          class="w-full text-xs font-mono"
          @update:model-value="(val) => updateProxyField('noProxy', String(val ?? ''))"
        />
        <span class="text-[11px] text-muted-color">
          Comma-separated list of hostnames, IP addresses, CIDR blocks, or domain suffixes
        </span>
      </div>
    </div>
  </div>
</template>
