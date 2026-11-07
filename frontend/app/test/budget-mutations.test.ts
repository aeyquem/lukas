import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada, useQueryCache } from '@pinia/colada'
import { retryPendingWrite, sendFinancialWrite } from '../src/shared/api/pending-commands'
import { budgetKeys } from '../src/features/budget/api/queries'
import { useBudgetRetryMutation, useBudgetWriteMutation } from '../src/features/budget/api/mutations'
import type { PendingFinancialWrite } from '../src/shared/api/pending-commands'

vi.mock('../src/shared/api/pending-commands', () => ({
  sendFinancialWrite: vi.fn(),
  retryPendingWrite: vi.fn(),
}))

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

it('preserves category write bytes and invalidates categories and budget snapshots', async () => {
  const receipt = { resource_id: 'category-1', revision: 4 }
  vi.mocked(sendFinancialWrite).mockResolvedValue(receipt)
  const cache = setup(useQueryCache)
  const invalidate = vi.spyOn(cache, 'invalidateQueries').mockResolvedValue([])
  const mutation = setup(useBudgetWriteMutation)
  const command = { method: 'PUT' as const, route: '/api/categories/category-1', body: '{"name":"Hogar"}' }

  await expect(mutation.mutateAsync(command)).resolves.toEqual(receipt)
  expect(sendFinancialWrite).toHaveBeenCalledExactlyOnceWith(command)
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: budgetKeys.categories })
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: budgetKeys.snapshots })
})

it('retries the saved command by idempotency key and invalidates snapshots and previews', async () => {
  vi.mocked(retryPendingWrite).mockResolvedValue({ resource_id: 'budget-2026-10', revision: 8 })
  const cache = setup(useQueryCache)
  const invalidate = vi.spyOn(cache, 'invalidateQueries').mockResolvedValue([])
  const mutation = setup(useBudgetRetryMutation)
  const savedCommand = { idempotencyKey: 'saved-key', route: '/api/budgets/2026-10/close' } as PendingFinancialWrite

  await expect(mutation.mutateAsync(savedCommand)).resolves.toEqual({ resource_id: 'budget-2026-10', revision: 8 })
  expect(retryPendingWrite).toHaveBeenCalledExactlyOnceWith('saved-key')
  expect(invalidate).toHaveBeenCalledWith({ key: budgetKeys.snapshots })
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: budgetKeys.closePreviews })
})
it('returns a confirmed receipt when a subsequent invalidation fails', async () => {
  const receipt = { resource_id: 'budget-2026-10', revision: 9 }
  vi.mocked(sendFinancialWrite).mockResolvedValue(receipt)
  const cache = setup(useQueryCache)
  vi.spyOn(cache, 'invalidateQueries').mockRejectedValue(new Error('refresh failed'))
  const mutation = setup(useBudgetWriteMutation)

  await expect(mutation.mutateAsync({ method: 'PUT', route: '/api/budgets/2026-10/assignments/category-1', body: '{}' })).resolves.toEqual(receipt)
})
