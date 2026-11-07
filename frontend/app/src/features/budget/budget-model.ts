import type { BudgetClosePreview, BudgetSnapshot, BudgetValue } from './api/contracts'
export type * from './api/contracts'

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

export function prepareAssignmentWrite(
  budget: BudgetSnapshot,
  categoryId: string,
  amount: string | undefined,
  reason: string,
): { categoryName: string; body: string; error?: string } | null {
  const category = budget.categories.find((item) => item.id === categoryId)
  if (!category || amount === undefined || !/^\d+(?:\.\d+)?$/.test(amount)) return null
  if (budget.closed && !reason.trim()) {
    return { categoryName: category.name, body: '', error: 'Indica el motivo de la corrección para un período cerrado.' }
  }
  return {
    categoryName: category.name,
    body: JSON.stringify({
      amount,
      expected_period_revision: budget.revision,
      expected_revision: category.assignment_revision,
      ...(budget.closed ? { reason: reason.trim() } : {}),
    }),
  }
}

export function prepareCloseWrite(
  preview: BudgetClosePreview,
  dispositions: Record<string, 'carry' | 'release'>,
): string | null {
  if (!preview.close.eligible) return null
  return JSON.stringify({
    expected_period_revision: preview.close.expected_period_revision,
    categories: preview.close.categories.map((item) => ({
      category_id: item.category_id,
      disposition: dispositions[item.category_id] ?? 'carry',
    })),
  })
}

export function usagePercent(category: BudgetSnapshot['categories'][number]): number | null {
  if (category.activity.state !== 'complete') return null
  const assigned = category.assigned
  const used = category.activity.amount ?? '0'
  const decimalParts = [assigned, used].map((value) => value.replace(/^-/, '').split('.'))
  const scale = Math.max(...decimalParts.map((parts) => parts[1]?.length ?? 0))
  const coefficients = [assigned, used].map((value) => {
    const negative = value.startsWith('-')
    const [whole, fraction = ''] = value.replace(/^-/, '').split('.')
    const coefficient = BigInt(`${whole}${fraction.padEnd(scale, '0')}`)
    return negative ? -coefficient : coefficient
  })
  if (coefficients[0] === 0n) return coefficients[1] > 0n ? 100 : 0
  if (coefficients[1] <= 0n) return 0
  const basisPoints = (coefficients[1] * 10_000n) / coefficients[0]
  return Number(basisPoints > 10_000n ? 10_000n : basisPoints) / 100
}

export function categoryOverused(category: BudgetSnapshot['categories'][number]): boolean {
  return category.remaining.state === 'complete' && (category.remaining.amount ?? '0').startsWith('-') && !/^-(?:0+)(?:\.0+)?$/.test(category.remaining.amount ?? '')
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
