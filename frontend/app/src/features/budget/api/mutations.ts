import { useMutation, useQueryCache } from '@pinia/colada'
import { retryPendingWrite, sendFinancialWrite, type FinancialWriteIntent, type PendingFinancialWrite } from '../../../shared/api/pending-commands'
import { invalidateFinancialQueries } from '../../../shared/api/finance-query-invalidation'
import type { BudgetGroupWrite, BudgetMutationReceipt, CategoryWrite } from './contracts'

/** Keep request serialization in one place; pending retries preserve these bytes. */
export function categoryWriteIntent(categoryId: string | null, fields: CategoryWrite): FinancialWriteIntent {
  return {
    method: categoryId ? 'PUT' : 'POST',
    route: categoryId ? `/api/categories/${encodeURIComponent(categoryId)}` : '/api/categories',
    body: JSON.stringify(fields),
  }
}

export function categoryDeleteIntent(categoryId: string, expectedRevision?: number): FinancialWriteIntent {
  return {
    method: 'DELETE',
    route: `/api/categories/${encodeURIComponent(categoryId)}`,
    body: JSON.stringify(expectedRevision === undefined ? {} : { expected_revision: expectedRevision }),
  }
}

export function groupWriteIntent(groupId: string | null, fields: BudgetGroupWrite): FinancialWriteIntent {
  return {
    method: groupId ? 'PUT' : 'POST',
    route: groupId ? `/api/budget-groups/${encodeURIComponent(groupId)}` : '/api/budget-groups',
    body: JSON.stringify(fields),
  }
}

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
