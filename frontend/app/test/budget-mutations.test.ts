import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { ApiClientError } from '../src/shared/api/client'
import { PiniaColada, useQueryCache } from '@pinia/colada'
import { retryPendingWrite, sendFinancialWrite } from '../src/shared/api/pending-commands'
import { budgetKeys } from '../src/features/budget/api/queries'
import { categoryDeleteIntent, categoryWriteIntent, groupWriteIntent, useBudgetRetryMutation, useBudgetWriteMutation } from '../src/features/budget/api/mutations'
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
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: budgetKeys.groups })
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

it('serializes category membership and ordering with an explicit expected revision', () => {
  expect(categoryWriteIntent('category-1', {
    name: 'Café', group_id: 'general', sort_order: 12, archived: false, expected_revision: 8,
  })).toEqual({
    method: 'PUT', route: '/api/categories/category-1',
    body: '{"name":"Café","group_id":"general","sort_order":12,"archived":false,"expected_revision":8}',
  })
  expect(categoryWriteIntent(null, { name: 'Nueva', sort_order: 0, group_id: 'savings' })).toEqual({
    method: 'POST', route: '/api/categories', body: '{"name":"Nueva","sort_order":0,"group_id":"savings"}',
  })
})

it('serializes group creation, archive, order, and revision without mutating the draft', () => {
  const fields = { name: 'Reserva', sort_order: 30, archived: true, expected_revision: 3 }
  expect(groupWriteIntent('group-1', fields)).toEqual({
    method: 'PUT', route: '/api/budget-groups/group-1', body: JSON.stringify(fields),
  })
  expect(groupWriteIntent(null, { name: 'Reserva', sort_order: 30 })).toEqual({
    method: 'POST', route: '/api/budget-groups', body: '{"name":"Reserva","sort_order":30}',
  })
  expect(fields.expected_revision).toBe(3)
})

it('sends guarded deletes through the durable write pipeline and preserves validation errors', async () => {
  const error = new ApiClientError({ kind: 'conflict', status: 409, code: 'category_in_use', message: 'La categoría conserva historia.' })
  vi.mocked(sendFinancialWrite).mockRejectedValueOnce(error)
  const cache = setup(useQueryCache)
  const invalidate = vi.spyOn(cache, 'invalidateQueries').mockResolvedValue([])
  const mutation = setup(useBudgetWriteMutation)
  const intent = categoryDeleteIntent('category-1', 5)
  expect(intent).toEqual({ method: 'DELETE', route: '/api/categories/category-1', body: '{"expected_revision":5}' })
  await expect(mutation.mutateAsync(intent)).rejects.toBe(error)
  expect(sendFinancialWrite).toHaveBeenCalledExactlyOnceWith(intent)
  expect(invalidate).not.toHaveBeenCalled()
  expect(categoryDeleteIntent('category-1').body).toBe('{}')
})
