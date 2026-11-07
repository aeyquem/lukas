import { useMutation, useQueryCache } from '@pinia/colada'
import { invalidateFinancialQueries } from '../../../shared/api/finance-query-invalidation'
import { retryPendingWrite, sendFinancialWrite, type FinancialWriteIntent, type PendingFinancialWrite } from '../../../shared/api/pending-commands'

export interface TransactionMutationReceipt {
  resource_id: string
  revision: number
}

function useTransactionInvalidation() {
  const cache = useQueryCache()
  return () => invalidateFinancialQueries(cache, 'transaction')
}

export function useTransactionWriteMutation() {
  const invalidate = useTransactionInvalidation()
  return useMutation({
    mutation: async (intent: FinancialWriteIntent) => {
      const receipt = await sendFinancialWrite<TransactionMutationReceipt>(intent)
      await invalidate()
      return receipt
    },
  })
}

export function useTransactionRetryMutation() {
  const invalidate = useTransactionInvalidation()
  return useMutation({
    mutation: async (command: PendingFinancialWrite) => {
      const receipt = await retryPendingWrite<TransactionMutationReceipt>(command.idempotencyKey)
      await invalidate()
      return receipt
    },
  })
}
