import { useMutation, useQueryCache } from '@pinia/colada'
import { retryPendingWrite, sendFinancialWrite, type FinancialWriteIntent, type PendingFinancialWrite } from '../../../shared/api/pending-commands'
import { invalidateFinancialQueries } from '../../../shared/api/finance-query-invalidation'
import type { BudgetMutationReceipt } from './contracts'

function useBudgetInvalidation() {
  const cache = useQueryCache()
  return () => invalidateFinancialQueries(cache, 'budget')
}

export function useBudgetWriteMutation() {
  const invalidate = useBudgetInvalidation()
  return useMutation({
    mutation: async (intent: FinancialWriteIntent) => {
      const receipt = await sendFinancialWrite<BudgetMutationReceipt>(intent)
      await invalidate()
      return receipt
    },
  })
}

export function useBudgetRetryMutation() {
  const invalidate = useBudgetInvalidation()
  return useMutation({
    mutation: async (command: PendingFinancialWrite) => {
      const receipt = await retryPendingWrite<BudgetMutationReceipt>(command.idempotencyKey)
      await invalidate()
      return receipt
    },
  })
}
