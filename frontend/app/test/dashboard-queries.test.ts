import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, nextTick, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada, useQueryCache } from '@pinia/colada'
import { apiRequest } from '../src/shared/api/client'
import { useAccountListQuery } from '../src/features/accounts/accounts-feature'
import { useBudgetSnapshotQuery, budgetKeys } from '../src/features/budget/budget-feature'
import { useTransactionPageQuery } from '../src/features/transactions/transactions-feature'
import { transactionKeys } from '../src/features/transactions/transactions-feature'
import type { BudgetSnapshot } from '../src/features/budget/budget-feature'
import type { TransactionPage } from '../src/features/transactions/transactions-feature'

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

it('composes public dashboard queries with period and recent-page cache identity', async () => {
  const snapshot: BudgetSnapshot = {
    period: '2026-10', revision: '1', funding: { state: 'complete', amount: '0' },
    ready_to_assign: { state: 'complete', amount: '0' }, activity: { state: 'complete', amount: '0' },
    uncategorized_activity: { state: 'complete', amount: '0' }, complete: true, categories: [],
  }
  const transactionPage: TransactionPage = { items: [], next: null }
  vi.mocked(apiRequest).mockImplementation(async (route) => route.startsWith('budgets/') ? snapshot : route.startsWith('transactions?') ? transactionPage : [])
  const period = ref('2026-10')
  const transactionEnabled = ref(false)
  const [accounts, budget, transactions, cache] = setup(() => [
    useAccountListQuery(), useBudgetSnapshotQuery(period), useTransactionPageQuery(null, 20, transactionEnabled, 30_000), useQueryCache(),
  ])

  await Promise.all([budget!.activate(), transactions!.refetch(true)])
  transactionEnabled.value = true
  await nextTick()
  expect(apiRequest).toHaveBeenCalledWith('budgets/2026-10')
  expect(apiRequest).toHaveBeenCalledWith('transactions?page_size=20')
  expect(cache!.getEntries().map((entry) => entry.key)).toContainEqual(budgetKeys.snapshot('2026-10'))
  expect(cache!.getEntries().map((entry) => entry.key)).toContainEqual(transactionKeys.page(20, null))
  const accountEntry = cache!.getEntries().find((entry) => entry.keyHash === JSON.stringify(['accounts', 'list']))
  expect(accountEntry?.options?.staleTime).toBe(30_000)

  budget!.suspend()
  await Promise.all([budget!.activate(), transactions!.refetch(true)])
  expect(apiRequest).toHaveBeenCalledTimes(5) // one account load and two explicit reads for each dashboard visit
})
