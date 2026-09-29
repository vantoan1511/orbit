export interface AppTab {
  id: string
  title: string
  route: string
  path: string
  query?: Record<string, string>
  iconName?: string
  category?: string
  closable?: boolean
}

export interface TabsStorageState {
  tabs: AppTab[]
  activeTabId: string | null
}
