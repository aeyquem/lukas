import { useQuery, type QueryCache } from '@pinia/colada'
import { ref, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { apiRequest } from '../../../shared/api/client'
import { budgetRoute } from '../budget-model'
import type { BudgetClosePreview, BudgetGroup, BudgetSnapshot, ExpenseCategory } from './contracts'

export const budgetKeys = {
  all: ['budget'] as const,
  snapshots: ['budget', 'period'] as const,
  snapshot: (period: string) => ['budget', 'period', period] as const,
  closePreviews: ['budget', 'close-preview'] as const,
  closePreview: (period: string) => ['budget', 'close-preview', period] as const,
  categories: ['expense-categories'] as const,
  groups: ['budget-groups'] as const,
}

export function budgetSnapshotQueryOptions(period: string) {
  return {
    key: budgetKeys.snapshot(period),
    query: () => apiRequest<BudgetSnapshot>(budgetRoute(period)),
    staleTime: 30_000,
    gcTime: 300_000,
    enabled: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  }
}

export async function fetchBudgetSnapshot(cache: QueryCache, period: string): Promise<BudgetSnapshot> {
  const options = { ...budgetSnapshotQueryOptions(period), enabled: true }
  const entry = cache.ensure<BudgetSnapshot>(options)
  const result = await cache.fetch(entry, options)
  if (result.status === 'success') return result.data
  if (result.status === 'error') throw result.error
  throw new Error('El resumen del presupuesto no está disponible.')
}

export function useBudgetSnapshotQuery(period: MaybeRefOrGetter<string>) {
  const enabled = ref(false)
  watch(() => toValue(period), () => { enabled.value = false }, { flush: 'sync' })
  const query = useQuery(() => ({ ...budgetSnapshotQueryOptions(toValue(period)), enabled: enabled.value }))
  async function refetchFresh(): Promise<BudgetSnapshot> {
    const result = await query.refetch(true)
    if (result.status === 'success') return result.data
    if (result.status === 'error') throw result.error
    throw new Error('El resumen del presupuesto no está disponible.')
  }
  async function activate(): Promise<BudgetSnapshot> {
    const snapshot = await refetchFresh()
    enabled.value = true
    return snapshot
  }
  function suspend(): void { enabled.value = false }
  function resume(): void { enabled.value = true }
  return { ...query, activate, refetchFresh, suspend, resume }
}

export function useExpenseCategoriesQuery() {
  const enabled = ref(false)
  const query = useQuery({
    key: budgetKeys.categories,
    query: () => apiRequest<ExpenseCategory[]>('categories'),
    staleTime: 30_000,
    gcTime: 300_000,
    enabled,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  })
  async function refetchFresh(): Promise<ExpenseCategory[]> {
    const result = await query.refetch(true)
    if (result.status === 'success') return result.data
    if (result.status === 'error') throw result.error
    throw new Error('No fue posible cargar las categorías.')
  }
  async function activate(): Promise<ExpenseCategory[]> {
    const categories = await refetchFresh()
    enabled.value = true
    return categories
  }
  return { ...query, activate, refetchFresh }
}

export function useBudgetGroupsQuery() {
  const enabled = ref(false)
  const query = useQuery({
    key: budgetKeys.groups,
    query: () => apiRequest<BudgetGroup[]>('budget-groups'),
    staleTime: 30_000,
    gcTime: 300_000,
    enabled,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  })
  async function refetchFresh(): Promise<BudgetGroup[]> {
    const result = await query.refetch(true)
    if (result.status === 'success') return result.data
    if (result.status === 'error') throw result.error
    throw new Error('No fue posible cargar los grupos.')
  }
  async function activate(): Promise<BudgetGroup[]> {
    const groups = await refetchFresh()
    enabled.value = true
    return groups
  }
  return { ...query, activate, refetchFresh }
}

export function useBudgetClosePreviewQuery(period: MaybeRefOrGetter<string>) {
  return useQuery(() => {
    const selectedPeriod = toValue(period)
    return {
      key: budgetKeys.closePreview(selectedPeriod),
      query: () => apiRequest<BudgetClosePreview>(budgetRoute(selectedPeriod, '/close-preview')),
      staleTime: 0,
      enabled: false,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    }
  })
}
