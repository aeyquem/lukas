import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { apiRequest } from '../src/shared/api/client'
import { useTransactionPages } from '../src/features/transactions/composables/use-transaction-pages'
import type { TransactionPage, TransactionView } from '../src/features/transactions/api/contracts'

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

function transaction(id: string): TransactionView {
  return {
    id,
    kind: 'expense',
    business_date: '2026-10-31',
    category_id: null,
    source_expense_id: null,
    payee: null,
    note: null,
    confirmed_clp_amount: null,
    conversion_evidence: null,
    revision: 1,
    effects: [],
  }
}

function page(id: string, next: TransactionPage['next']): TransactionPage {
  return { items: [transaction(id)], next }
}

it('appends cursor pages in server order', async () => {
  const cursor = { before_date: '2026-10-31', before_id: 'transaction-1' }
  vi.mocked(apiRequest)
    .mockResolvedValueOnce(page('transaction-1', cursor))
    .mockResolvedValueOnce(page('transaction-2', null))
  const pages = setup(useTransactionPages)

  await pages.loadFirstPage()
  await pages.loadNextPage()

  expect(pages.items.value.map((item) => item.id)).toEqual(['transaction-1', 'transaction-2'])
  expect(apiRequest).toHaveBeenNthCalledWith(1, 'transactions?page_size=50')
  expect(apiRequest).toHaveBeenNthCalledWith(
    2,
    'transactions?page_size=50&before_date=2026-10-31&before_id=transaction-1',
  )
  expect(pages.nextCursor.value).toBeNull()
})

it('ignores an older next-page response after a first-page refresh', async () => {
  const cursor = { before_date: '2026-10-31', before_id: 'transaction-1' }
  let finishNext!: (value: TransactionPage) => void
  vi.mocked(apiRequest)
    .mockResolvedValueOnce(page('transaction-1', cursor))
    .mockImplementationOnce(() => new Promise((resolve) => { finishNext = resolve }))
    .mockResolvedValueOnce(page('fresh-transaction-1', cursor))
  const pages = setup(useTransactionPages)

  await pages.loadFirstPage()
  const oldNextPage = pages.loadNextPage()
  await pages.loadFirstPage()
  finishNext(page('stale-transaction-2', null))
  await oldNextPage

  expect(pages.items.value.map((item) => item.id)).toEqual(['fresh-transaction-1'])
  expect(pages.isLoadingMore.value).toBe(false)
})
