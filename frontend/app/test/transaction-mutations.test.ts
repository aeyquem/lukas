import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada, useQueryCache } from '@pinia/colada'
import { retryPendingWrite, sendFinancialWrite } from '../src/shared/api/pending-commands'
import { transactionKeys } from '../src/features/transactions/api/queries'
import { useTransactionRetryMutation, useTransactionWriteMutation } from '../src/features/transactions/api/mutations'
import { transactionFieldMessage } from '../src/features/transactions/transaction-field-message'
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

it('returns a confirmed write receipt even when query refresh fails', async () => {
  const receipt = { resource_id: 'transaction-1', revision: 4 }
  vi.mocked(sendFinancialWrite).mockResolvedValue(receipt)
  const cache = setup(useQueryCache)
  const invalidate = vi.spyOn(cache, 'invalidateQueries').mockRejectedValue(new Error('refresh failed'))
  const mutation = setup(useTransactionWriteMutation)
  const command = { method: 'PUT' as const, route: '/api/transactions/transaction-1', body: '{"note":"updated"}' }

  await expect(mutation.mutateAsync(command)).resolves.toEqual(receipt)
  expect(sendFinancialWrite).toHaveBeenCalledExactlyOnceWith(command)
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: transactionKeys.pages })
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: ['accounts'] })
  expect(invalidate.mock.calls.map(([filter]) => filter)).toContainEqual({ key: ['budget', 'period'] })
})

it('does not retry a rejected write automatically and retries a saved command by key', async () => {
  const failure = new Error('unknown outcome')
  vi.mocked(sendFinancialWrite).mockRejectedValue(failure)
  const cache = setup(useQueryCache)
  const invalidate = vi.spyOn(cache, 'invalidateQueries').mockResolvedValue([])
  const writeMutation = setup(useTransactionWriteMutation)
  await expect(writeMutation.mutateAsync({ method: 'POST', route: '/api/transactions', body: '{}' })).rejects.toBe(failure)
  expect(sendFinancialWrite).toHaveBeenCalledOnce()
  expect(retryPendingWrite).not.toHaveBeenCalled()
  expect(invalidate).not.toHaveBeenCalled()

  const savedCommand = { idempotencyKey: 'saved-key', route: '/api/transactions' } as PendingFinancialWrite
  vi.mocked(retryPendingWrite).mockResolvedValue({ resource_id: 'transaction-1', revision: 5 })
  const retryMutation = setup(useTransactionRetryMutation)
  await expect(retryMutation.mutateAsync(savedCommand)).resolves.toEqual({ resource_id: 'transaction-1', revision: 5 })
  expect(retryPendingWrite).toHaveBeenCalledExactlyOnceWith('saved-key')
  expect(invalidate).toHaveBeenCalled()
})

it('keeps transaction field validation messages owned by the feature', () => {
  expect(transactionFieldMessage('required_for_opening_date')).toBe('Elige cómo tratar el movimiento respecto del saldo inicial.')
  expect(transactionFieldMessage('unrecognized_error')).toBe('Revisa este campo.')
})
