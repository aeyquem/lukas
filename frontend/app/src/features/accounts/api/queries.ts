import { defineQuery, useQuery } from '@pinia/colada'
import { toValue, type MaybeRefOrGetter } from 'vue'
import { apiRequest } from '../../../shared/api/client'
import type { AccountView } from './contracts'

export const accountKeys = {
  all: ['accounts'] as const,
  list: ['accounts', 'list'] as const,
  detail: (id: string) => ['accounts', 'detail', id] as const,
}
import { validAccountId } from './account-id'
const freshness = { staleTime: 30_000, refetchOnWindowFocus: false, refetchOnReconnect: false } as const
export const useAccountListQuery = defineQuery({
  key: accountKeys.list,
  query: () => apiRequest<AccountView[]>('accounts'),
  ...freshness,
})
export function useAccountDetailQuery(id: MaybeRefOrGetter<string | null | undefined>) {
  return useQuery(() => {
    const accountId = toValue(id) ?? ''
    return {
      key: accountKeys.detail(accountId),
      enabled: validAccountId(accountId),
      query: () => apiRequest<AccountView>(`accounts/${encodeURIComponent(accountId)}`),
      ...freshness,
    }
  })
}
export { useAccountActivityQuery } from '../../transactions/api/queries'
