import { useQuery } from '@pinia/colada'
import { toValue, type MaybeRefOrGetter } from 'vue'
import { apiRequest } from '../../../shared/api/client'
import { validAccountId } from '../../accounts/api/account-id'
import type { AccountTransactionPage } from './contracts'
const freshness = { staleTime: 30_000, refetchOnWindowFocus: false, refetchOnReconnect: false } as const
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
