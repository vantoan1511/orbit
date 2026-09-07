import { storage } from './nativeService.ts'
import type { PersistedTableFilters } from '../types/tableFilter.ts'
import { isAllowedRowOption } from '../types/tableFilter.ts'

export const TABLE_FILTER_STORAGE_KEY = 'orbit_table_filter_preferences'
const STORAGE_TIMEOUT_MS = 1500

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | null = null
  const timeoutPromise = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(`Storage operation timed out after ${ms}ms`)), ms)
  })
  return Promise.race([promise, timeoutPromise]).finally(() => {
    if (timer) clearTimeout(timer)
  })
}

function sanitizeObjectRecord<T>(record: unknown): Record<string, T> {
  if (!record || typeof record !== 'object' || Array.isArray(record)) {
    return {}
  }
  const result: Record<string, T> = {}
  for (const [key, value] of Object.entries(record)) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      result[key] = value as T
    }
  }
  return result
}

export function createDefaultFilters(): PersistedTableFilters {
  return {
    version: 1,
    defaultRows: 25,
    tables: {},
    clusters: {}
  }
}

export function validatePersistedFilters(parsed: unknown): PersistedTableFilters {
  if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
    const candidate = parsed as Partial<PersistedTableFilters>
    const defaultRowsCandidate = candidate.defaultRows
    const isValidDefaultRows = isAllowedRowOption(defaultRowsCandidate)

    return {
      version: typeof candidate.version === 'number' ? candidate.version : 1,
      defaultRows: isValidDefaultRows ? defaultRowsCandidate : 25,
      tables: sanitizeObjectRecord(candidate.tables),
      clusters: sanitizeObjectRecord(candidate.clusters)
    }
  }
  return createDefaultFilters()
}

export const tableFilterService = {
  /**
   * Load persisted table filters from native desktop storage,
   * falling back to a one-time migration from legacy localStorage if present.
   */
  async loadFilters(): Promise<PersistedTableFilters> {
    try {
      let raw: string | null = null

      try {
        if (typeof window !== 'undefined' && !(window as unknown as { NL_PORT?: number }).NL_PORT) {
          throw new Error('Neutralino runtime not available')
        }
        raw = await withTimeout(storage.getData(TABLE_FILTER_STORAGE_KEY), STORAGE_TIMEOUT_MS)
      } catch {
        // Native storage key not found or uninitialized runtime.
        // Fall back to check for legacy localStorage data
        if (typeof localStorage !== 'undefined') {
          try {
            const legacy = localStorage.getItem(TABLE_FILTER_STORAGE_KEY)
            if (legacy) {
              raw = legacy
              // Migrate to native desktop storage and clean up legacy localStorage key
              try {
                await storage.setData(TABLE_FILTER_STORAGE_KEY, legacy)
                localStorage.removeItem(TABLE_FILTER_STORAGE_KEY)
              } catch (migrationErr) {
                console.warn(
                  'Failed to write migrated table filters to native storage:',
                  migrationErr
                )
              }
            }
          } catch {
            // localStorage not accessible or restricted
          }
        }
      }

      if (raw) {
        const parsed = JSON.parse(raw)
        return validatePersistedFilters(parsed)
      }
    } catch (e) {
      console.warn('Failed to load table filters from native storage:', e)
    }

    return createDefaultFilters()
  },

  /**
   * Persist table filters to native desktop storage.
   */
  async saveFilters(filters: PersistedTableFilters): Promise<void> {
    try {
      await storage.setData(TABLE_FILTER_STORAGE_KEY, JSON.stringify(filters))
    } catch (e) {
      console.warn('Failed to save table filters to native storage:', e)
    }
  },

  /**
   * Remove persisted table filters from native desktop storage.
   */
  async clearFilters(): Promise<void> {
    try {
      await storage.removeData(TABLE_FILTER_STORAGE_KEY)
    } catch (e) {
      console.warn('Failed to remove table filters from native storage:', e)
    }
  }
}
