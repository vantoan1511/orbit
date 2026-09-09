import type { Component } from 'vue'

export type SearchCategory = 'all' | 'resources' | 'logs' | 'navigation'

export interface SearchResultItem {
  id: string
  title: string
  subtitle?: string
  category: 'resources' | 'logs' | 'navigation'
  kind?: string
  namespace?: string
  status?: string
  icon?: Component
  iconColorClass?: string
  score?: number
  action: () => void | Promise<void>
}

export type SearchCategoryCounts = Record<SearchCategory, number>
