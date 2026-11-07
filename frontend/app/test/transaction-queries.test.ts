import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, nextTick, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada, useQueryCache } from '@pinia/colada'
import { apiRequest } from '../src/shared/api/client'
import { transactionKeys, useTransactionPageQuery } from '../src/features/transactions/api/queries'
import type { TransactionCursor, TransactionPage } from '../src/features/transactions/api/contracts'

vi.mock('../src/shared/api/client', () => ({ apiRequest: vi.fn() }))

let app: ReturnType<typeof createApp>

beforeEach(() => {
  vi.resetAllMocks()
  const pinia = createPinia()
  setActivePinia(pinia)
  app = createApp({})
  app.use(pinia)
  app.use(PiniaColada, { pinia })
})

function setup<T>(fn: () => T): T {
  return app.runWithContext(() => effectScope().run(fn)!)
}

it('keys ledger pages by page size and both cursor fields', async () => {
  vi.mocked(apiRequest).mockResolvedValue({ items: [], next: null } satisfies TransactionPage)
  const cursor = ref<TransactionCursor | null>(null)
  const [query, cache] = setup(() => [useTransactionPageQuery(cursor), useQueryCache()])

  await query!.refresh(true)
  expect(apiRequest).toHaveBeenLastCalledWith('transactions?page_size=50')

  cursor.value = { before_date: '2026-10-31', before_id: 'transaction-50' }
  await nextTick()
  await query!.refresh(true)

  expect(apiRequest).toHaveBeenLastCalledWith(
    'transactions?page_size=50&before_date=2026-10-31&before_id=transaction-50',
  )
  const keys = cache!.getEntries().map((entry) => entry.key)
  expect(keys).toContainEqual(transactionKeys.page(50, null))
  expect(keys).toContainEqual(transactionKeys.page(50, cursor.value))
  expect(keys).not.toContainEqual(transactionKeys.page(100, cursor.value))
})

it('requires every route mount to fetch a fresh ledger page', async () => {
  vi.mocked(apiRequest).mockResolvedValue({ items: [], next: null } satisfies TransactionPage)
  const [query, cache] = setup(() => [useTransactionPageQuery(null, 20), useQueryCache()])
  await query!.refresh(true)

  const entry = cache!.getEntries().find((item) => item.keyHash === JSON.stringify(transactionKeys.page(20, null)))
  expect(entry?.options?.refetchOnMount).toBe('always')
  expect(entry?.options?.staleTime).toBe(0)
  expect(entry?.options?.refetchOnWindowFocus).toBe(false)
  expect(entry?.options?.refetchOnReconnect).toBe(false)
})
