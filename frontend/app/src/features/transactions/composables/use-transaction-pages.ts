import { useQueryCache } from '@pinia/colada'
import { ref, shallowRef } from 'vue'
import { transactionPageQueryOptions } from '../api/queries'
import type { TransactionCursor, TransactionPage, TransactionView } from '../api/contracts'

const pageSize = 50

/** Owns the loaded ledger pages while Colada owns each fetched page snapshot. */
export function useTransactionPages() {
  const cache = useQueryCache()
  const items = shallowRef<TransactionView[]>([])
  const nextCursor = ref<TransactionCursor | null>(null)
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const error = shallowRef<unknown>(null)
  let generation = 0

  async function fetchPage(cursor: TransactionCursor | null): Promise<TransactionPage> {
    const options = transactionPageQueryOptions(pageSize, cursor)
    const entry = cache.ensure<TransactionPage>(options)
    const result = await cache.fetch(entry, options)
    if (result.status === 'success') return result.data
    if (result.status === 'error') throw result.error
    throw new Error('La respuesta de movimientos no está disponible.')
  }

  async function loadFirstPage(): Promise<void> {
    const requestGeneration = ++generation
    isLoading.value = true
    isLoadingMore.value = false
    error.value = null
    try {
      const page = await fetchPage(null)
      if (requestGeneration !== generation) return
      items.value = page.items
      nextCursor.value = page.next
    } catch (cause) {
      if (requestGeneration === generation) error.value = cause
      throw cause
    } finally {
      if (requestGeneration === generation) isLoading.value = false
    }
  }

  async function loadNextPage(): Promise<void> {
    const cursor = nextCursor.value
    if (!cursor || isLoadingMore.value) return
    const requestGeneration = generation
    isLoadingMore.value = true
    error.value = null
    try {
      const page = await fetchPage(cursor)
      if (requestGeneration !== generation || cursor !== nextCursor.value) return
      items.value = [...items.value, ...page.items]
      nextCursor.value = page.next
    } catch (cause) {
      if (requestGeneration === generation) error.value = cause
      throw cause
    } finally {
      if (requestGeneration === generation) isLoadingMore.value = false
    }
  }

  return { items, nextCursor, isLoading, isLoadingMore, error, loadFirstPage, loadNextPage }
}
