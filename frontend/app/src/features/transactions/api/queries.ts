import { defineQuery, useQuery } from '@pinia/colada'
import { toValue, type MaybeRefOrGetter } from 'vue'
import { apiRequest } from '../../../shared/api/client'
import { validAccountId } from '../../accounts/api/account-id'
import type { AccountTransactionPage, TransactionCategory, TransactionCursor, TransactionPage } from './contracts'
const freshness = { staleTime: 30_000, refetchOnWindowFocus: false, refetchOnReconnect: false } as const

export const transactionKeys = {
  all: ['transactions'] as const,
  pages: ['transactions', 'page'] as const,
  categories: ['transactions', 'categories'] as const,
  page: (pageSize: number, cursor: TransactionCursor | null) => [
    'transactions',
    'page',
    pageSize,
    cursor?.before_date ?? '',
    cursor?.before_id ?? '',
  ] as const,
}

interface BootstrapResponse { categories: TransactionCategory[] }

export const useTransactionCategoriesQuery = defineQuery({
  key: transactionKeys.categories,
  query: async () => (await apiRequest<BootstrapResponse>('bootstrap')).categories,
  staleTime: 0,
  // The route loads bootstrap alongside accounts in one explicit initialization.
  refetchOnMount: false,
  refetchOnWindowFocus: false,
  refetchOnReconnect: false,
})

export interface ActivityCursor { before_date: string; before_id: string }
export function useAccountActivityQuery(id: MaybeRefOrGetter<string | null | undefined>, cursor: MaybeRefOrGetter<ActivityCursor | null> = null, pageSize = 50) {
  return useQuery(() => {
    const accountId = toValue(id) ?? ''
    const page = toValue(cursor)
    const params = new URLSearchParams({ account_id: accountId, page_size: String(pageSize) })
    if (page) { params.set('before_date', page.before_date); params.set('before_id', page.before_id) }
    return {
      key: ['account-activity', accountId, pageSize, page?.before_date ?? '', page?.before_id ?? ''],
      enabled: validAccountId(accountId),
      query: () => apiRequest<AccountTransactionPage>(`transactions?${params}`),
      ...freshness,
    }
  })
}

export function transactionPageQueryOptions(pageSize: number, cursor: TransactionCursor | null) {
  const params = new URLSearchParams({ page_size: String(pageSize) })
  if (cursor) {
    params.set('before_date', cursor.before_date)
    params.set('before_id', cursor.before_id)
  }
  return {
    key: transactionKeys.page(pageSize, cursor),
    query: () => apiRequest<TransactionPage>(`transactions?${params}`),
    staleTime: 0,
    gcTime: 300_000,
    enabled: true,
    refetchOnMount: 'always' as const,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  }
}

/** Query one deterministic page from the workspace transaction ledger. */
export function useTransactionPageQuery(
  cursor: MaybeRefOrGetter<TransactionCursor | null> = null,
  pageSize = 50,
  enabled: MaybeRefOrGetter<boolean> = true,
  staleTime = 0,
) {
  return useQuery(() => ({ ...transactionPageQueryOptions(pageSize, toValue(cursor)), enabled: toValue(enabled), staleTime }))
}
