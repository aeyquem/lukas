import { onScopeDispose } from 'vue'

// Temporary notification bridge for mounted readers that do not yet use Colada.
// It holds no data. Each reader retains its request guards and local drafts.
const readers = new Set<() => void | Promise<void>>()
export function onFinanceInvalidated(refresh: () => void | Promise<void>): void {
  readers.add(refresh)
  onScopeDispose(() => readers.delete(refresh))
}
export function notifyFinanceInvalidated(): void {
  for (const refresh of readers) {
    try { void Promise.resolve(refresh()).catch(() => {}) } catch { /* Reader owns its visible error. */ }
  }
}
