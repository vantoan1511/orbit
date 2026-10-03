import { ENABLED_SETTINGS_TABS, type SettingsSubTab } from '../types/settings.ts'

export function resolveSettingsTab(queryTab: unknown): SettingsSubTab {
  if (
    typeof queryTab === 'string' &&
    (ENABLED_SETTINGS_TABS as readonly string[]).includes(queryTab)
  ) {
    return queryTab as SettingsSubTab
  }
  return 'general'
}
