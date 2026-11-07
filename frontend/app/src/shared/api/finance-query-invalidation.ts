import type { QueryCache } from '@pinia/colada'
import { accountKeys } from '../../features/accounts/api/queries'
import { transactionKeys } from '../../features/transactions/api/queries'
import { budgetKeys } from '../../features/budget/api/queries'

export type FinanceWriteScope = 'account' | 'transaction' | 'budget'

/** Invalidates the server snapshots affected by an accepted financial write. */
export async function invalidateFinancialQueries(cache: QueryCache, scope: FinanceWriteScope): Promise<void> {
  const keys = scope === 'budget'
    ? [budgetKeys.categories, budgetKeys.snapshots, budgetKeys.closePreviews]
    : [
        accountKeys.all,
        ['account-activity'],
        transactionKeys.pages,
        budgetKeys.snapshots,
        budgetKeys.closePreviews,
      ]
  await Promise.allSettled(keys.map((key) => cache.invalidateQueries({ key })))
}
