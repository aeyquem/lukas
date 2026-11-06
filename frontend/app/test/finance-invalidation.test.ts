import { expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import { notifyFinanceInvalidated, onFinanceInvalidated } from '../src/shared/api/finance-invalidation'

it('refreshes independent readers despite failures and removes subscriptions with their scope', async () => {
  const failed = vi.fn(() => Promise.reject(new Error('read unavailable')))
  const refreshed = vi.fn()
  const scope = effectScope()
  scope.run(() => {
    onFinanceInvalidated(failed)
    onFinanceInvalidated(refreshed)
  })
  notifyFinanceInvalidated()
  await Promise.resolve()
  expect(failed).toHaveBeenCalledTimes(1)
  expect(refreshed).toHaveBeenCalledTimes(1)
  scope.stop()
  notifyFinanceInvalidated()
  expect(failed).toHaveBeenCalledTimes(1)
  expect(refreshed).toHaveBeenCalledTimes(1)
})
