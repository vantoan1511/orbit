export const DEFAULT_STORAGE_TIMEOUT_MS = 1500

/**
 * Wraps a promise with a timeout. If the promise does not settle within `ms` milliseconds,
 * it rejects with a timeout error and cleans up the timer.
 */
export function withTimeout<T>(
  promise: Promise<T>,
  ms: number = DEFAULT_STORAGE_TIMEOUT_MS,
  errorMessage?: string
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | null = null
  const timeoutPromise = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () => reject(new Error(errorMessage ?? `Storage operation timed out after ${ms}ms`)),
      ms
    )
  })
  return Promise.race([promise, timeoutPromise]).finally(() => {
    if (timer) clearTimeout(timer)
  })
}
