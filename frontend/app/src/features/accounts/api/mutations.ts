import { useMutation, useQueryCache } from '@pinia/colada'
import { apiRequest } from '../../../shared/api/client'
import { sendFinancialWrite, retryPendingWrite, type PendingFinancialWrite } from '../../../shared/api/pending-commands'
import { notifyFinanceInvalidated } from '../../../shared/api/finance-invalidation'
import { writeThenRead } from './write-flow'
import { accountKeys } from './queries'
import type { AccountView, MutationReceipt } from './contracts'

export function useFinanceInvalidation() {
  const cache = useQueryCache()
  return async () => {
    notifyFinanceInvalidated()
    await Promise.allSettled([
      cache.invalidateQueries({ key: accountKeys.all }),
      cache.invalidateQueries({ key: ['account-activity'] }),
    ])
  }
}
function useAccountReadback() {
  const cache = useQueryCache()
  const invalidate = useFinanceInvalidation()
  return async (write: () => Promise<MutationReceipt>, route: string) => {
    const outcome = await writeThenRead(write, async (receipt) => {
      const id = route === '/api/accounts' ? receipt.resource_id : route.split('/')[3]!
      return apiRequest<AccountView>(`accounts/${encodeURIComponent(id)}`)
    })
    if (outcome.value) cache.setQueryData(accountKeys.detail(outcome.value.id), outcome.value)
    await invalidate()
    return outcome
  }
}
export function useAccountWriteMutation() {
  const readback = useAccountReadback()
  return useMutation({
    mutation: (intent: { method: 'POST' | 'PUT'; route: string; payload: unknown }) => readback(
      () => sendFinancialWrite<MutationReceipt>({
        method: intent.method,
        route: intent.route,
        body: JSON.stringify(intent.payload),
      }),
      intent.route,
    ),
  })
}

export function useAccountRetryMutation() {
  const readback = useAccountReadback()
  return useMutation({
    mutation: (command: PendingFinancialWrite) => readback(
      () => retryPendingWrite<MutationReceipt>(command.idempotencyKey),
      command.route,
    ),
  })
}
