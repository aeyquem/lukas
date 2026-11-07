import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, nextTick, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada, useQueryCache } from '@pinia/colada'
import { apiRequest } from '../src/shared/api/client'
import {
  budgetKeys,
  fetchBudgetSnapshot,
  useBudgetClosePreviewQuery,
  useBudgetSnapshotQuery,
  useExpenseCategoriesQuery,
} from '../src/features/budget/api/queries'
import type { BudgetClosePreview, BudgetSnapshot, ExpenseCategory } from '../src/features/budget/api/contracts'

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

const snapshot = (period: string): BudgetSnapshot => ({
  period,
  revision: '1',
  funding: { state: 'complete', amount: '0' },
  ready_to_assign: { state: 'complete', amount: '0' },
  activity: { state: 'complete', amount: '0' },
  uncategorized_activity: { state: 'complete', amount: '0' },
  complete: true,
  categories: [],
})

it('keys snapshots by URL-owned period and forces route-entry refreshes', async () => {
  vi.mocked(apiRequest).mockImplementation(async (route) => snapshot(route.split('/').at(-1)!))
  const period = ref('2026-10')
  const [query, cache] = setup(() => [useBudgetSnapshotQuery(period), useQueryCache()])

  await query!.activate()
  expect(apiRequest).toHaveBeenLastCalledWith('budgets/2026-10')
  query!.suspend()
  period.value = '2026-11'
  await nextTick()
  await query!.activate()
  expect(apiRequest).toHaveBeenLastCalledWith('budgets/2026-11')
  expect(cache!.getEntries().map((entry) => entry.key)).toContainEqual(budgetKeys.snapshot('2026-10'))
  expect(cache!.getEntries().map((entry) => entry.key)).toContainEqual(budgetKeys.snapshot('2026-11'))
  const currentEntry = cache!.getEntries().find((entry) => entry.keyHash === JSON.stringify(budgetKeys.snapshot('2026-11')))
  expect(currentEntry?.options?.enabled).toBe(true)
  expect(currentEntry?.options?.staleTime).toBe(30_000)
})

it('queries expense categories and loads close previews only on explicit request', async () => {
  const categories: ExpenseCategory[] = [{ id: 'category-1', slug: 'home', name: 'Hogar', sort_order: 10, archived: false }]
  vi.mocked(apiRequest).mockImplementation(async (route) => route === 'categories' ? categories : ({ ...snapshot('2026-10'), close: { eligible: true, reason: null, expected_period_revision: '1', categories: [] } } satisfies BudgetClosePreview))
  const [categoryQuery, previewQuery] = setup(() => [useExpenseCategoriesQuery(), useBudgetClosePreviewQuery('2026-10')])
  expect(apiRequest).not.toHaveBeenCalledWith('budgets/2026-10/close-preview')

  await categoryQuery!.activate()
  expect(categoryQuery!.data.value).toEqual(categories)
  expect(apiRequest).toHaveBeenLastCalledWith('categories')

  await previewQuery!.refetch(true)
  expect(apiRequest).toHaveBeenLastCalledWith('budgets/2026-10/close-preview')
})

it('forces a fresh read for the same period after re-entry and rejects failed reads', async () => {
  vi.mocked(apiRequest)
    .mockResolvedValueOnce(snapshot('2026-10'))
    .mockRejectedValueOnce(new Error('offline'))
  const query = setup(() => useBudgetSnapshotQuery('2026-10'))
  await query.activate()
  query.suspend()
  await expect(query.activate()).rejects.toThrow('offline')
  expect(apiRequest).toHaveBeenCalledTimes(2)
})

it('resumes the next-period observer after close readback for later invalidations', async () => {
  vi.mocked(apiRequest).mockImplementation(async (route) => snapshot(route.split('/').at(-1)!))
  const period = ref('2026-10')
  const [query, cache] = setup(() => [useBudgetSnapshotQuery(period), useQueryCache()])
  await query!.activate()
  query!.suspend()
  period.value = '2026-11'
  await nextTick()
  await fetchBudgetSnapshot(cache!, '2026-11')
  query!.resume()
  await nextTick()

  await cache!.invalidateQueries({ key: budgetKeys.snapshot('2026-11') })
  expect(apiRequest).toHaveBeenCalledTimes(3)
  expect(apiRequest).toHaveBeenLastCalledWith('budgets/2026-11')
})
