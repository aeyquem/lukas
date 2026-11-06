export interface BudgetValue {
  state: 'complete' | 'unavailable' | 'incomplete'
  amount?: string
  known_subtotal?: string
  reasons?: string[]
  unresolved?: Array<{ kind: string; category_id?: string | null; currency?: string | null; amount?: string | null }>
}

export interface BudgetCategory {
  id: string
  name: string
  sort_order: number
  assigned: string
  assignment_revision: number | null
  assignment_intent?: string | null
  activity: BudgetValue
  remaining: BudgetValue
}

export interface BudgetSnapshot {
  period: string
  revision: string
  funding: BudgetValue
  ready_to_assign: BudgetValue
  activity: BudgetValue
  uncategorized_activity: BudgetValue
  complete: boolean
  closed?: boolean
  categories: BudgetCategory[]
}

export function currentPeriodInSantiago(now = new Date()): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: 'America/Santiago', year: 'numeric', month: '2-digit',
  }).formatToParts(now)
  const value = (type: string) => parts.find((part) => part.type === type)?.value ?? ''
  return `${value('year')}-${value('month')}`
}

export function budgetRoute(period: string, suffix = ''): string {
  return `budgets/${period}${suffix}`
}

export function normalizeCategoryDraft(
  name: string,
  sortOrder: string,
): { name: string; sort_order: number } | null {
  const normalizedName = name.trim()
  const normalizedOrder = Number(sortOrder)
  if (
    !normalizedName ||
    !Number.isInteger(normalizedOrder) ||
    normalizedOrder < 0 ||
    normalizedOrder > 32767
  ) return null
  return { name: normalizedName, sort_order: normalizedOrder }
}

export function describeBudgetValue(value: BudgetValue): string {
  if (value.state === 'complete') return value.amount ?? '0'
  if (value.state === 'unavailable') return 'No disponible'
  return `${value.known_subtotal ?? '0'} (subtotal conocido)`
}

export function unresolvedDescriptions(value: BudgetValue): string[] {
  return (value.unresolved ?? []).map((item) => {
    const detail = [item.amount, item.currency].filter(Boolean).join(' ')
    return detail ? `${item.kind}: ${detail}` : item.kind
  })
}
