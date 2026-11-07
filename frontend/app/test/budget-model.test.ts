import { describe, expect, it } from 'vitest'
import { budgetRoute, categoryOverused, currentPeriodInSantiago, describeBudgetValue, normalizeCategoryDraft, prepareAssignmentWrite, prepareCloseWrite, unresolvedDescriptions, usagePercent, type BudgetValue } from '../src/features/budget/budget-model'

describe('budget display model', () => {
  it('uses the current calendar month in America/Santiago', () => {
    expect(currentPeriodInSantiago(new Date('2026-11-01T02:30:00.000Z'))).toBe('2026-10')
    expect(currentPeriodInSantiago(new Date('2026-11-01T04:30:00.000Z'))).toBe('2026-11')
  })

  it('builds a period route from the month string rather than a ref wrapper', () => {
    expect(budgetRoute('2026-11')).toBe('budgets/2026-11')
    expect(budgetRoute('2026-11', '/close-preview')).toBe('budgets/2026-11/close-preview')
  })

  it('trims category names and enforces the backend nonnegative smallint ordering', () => {
    expect(normalizeCategoryDraft('  Hogar  ', '20')).toEqual({ name: 'Hogar', sort_order: 20 })
    expect(normalizeCategoryDraft('Transporte', '0')).toEqual({ name: 'Transporte', sort_order: 0 })
    expect(normalizeCategoryDraft('x', '-1')).toBeNull()
    expect(normalizeCategoryDraft('x', '32768')).toBeNull()
    expect(normalizeCategoryDraft(' ', '1')).toBeNull()
  })

  it('builds revision-checked assignment writes and requires closed-period reasons', () => {
    const budget = {
      period: '2026-10', revision: '12',
      funding: { state: 'complete' as const, amount: '10' },
      ready_to_assign: { state: 'complete' as const, amount: '10' },
      activity: { state: 'complete' as const, amount: '0' },
      uncategorized_activity: { state: 'complete' as const, amount: '0' },
      complete: true,
      categories: [{ id: 'category-1', name: 'Hogar', sort_order: 10, assigned: '10', assignment_revision: 7, activity: { state: 'complete' as const, amount: '0' }, remaining: { state: 'complete' as const, amount: '10' } }],
    }
    expect(prepareAssignmentWrite(budget, 'category-1', '15.50', '')).toEqual({
      categoryName: 'Hogar',
      body: '{"amount":"15.50","expected_period_revision":"12","expected_revision":7}',
    })
    const closedBudget = { ...budget, closed: true }
    expect(prepareAssignmentWrite(closedBudget, 'category-1', '15.50', '   ')?.error).toBe('Indica el motivo de la corrección para un período cerrado.')
    expect(prepareAssignmentWrite(closedBudget, 'category-1', '15.50', 'corrección')?.body).toContain('"reason":"corrección"')
  })

  it('does not imply a partial value is complete or unavailable', () => {
    const incomplete: BudgetValue = {
      state: 'incomplete',
      known_subtotal: '1250.50',
      unresolved: [{ kind: 'missing_rate', currency: 'USD', amount: '3.25' }],
    }
    expect(describeBudgetValue(incomplete)).toBe('1250.50 (subtotal conocido)')
    expect(unresolvedDescriptions(incomplete)).toEqual(['missing_rate: 3.25 USD'])
  })

  it('does not expose a partial amount for unavailable totals', () => {
    const unavailable: BudgetValue = { state: 'unavailable', reasons: ['account_not_eligible'] }
    expect(describeBudgetValue(unavailable)).toBe('No disponible')
    expect(unresolvedDescriptions(unavailable)).toEqual([])
  })

  it('builds a close command from the preview revision and selected dispositions', () => {
    const preview = {
      period: '2026-10', revision: '1', funding: { state: 'complete' as const }, ready_to_assign: { state: 'complete' as const },
      activity: { state: 'complete' as const }, uncategorized_activity: { state: 'complete' as const }, complete: true,
      categories: [], close: { eligible: true, reason: null, expected_period_revision: '15', categories: [
        { category_id: 'category-1', proposed_disposition: 'carry' as const },
        { category_id: 'category-2', proposed_disposition: 'carry' as const },
      ] },
    }
    expect(prepareCloseWrite(preview, { 'category-1': 'release' })).toBe(
      '{"expected_period_revision":"15","categories":[{"category_id":"category-1","disposition":"release"},{"category_id":"category-2","disposition":"carry"}]}',
    )
    expect(prepareCloseWrite({ ...preview, close: { ...preview.close, eligible: false } }, {})).toBeNull()
  })

  it('calculates usage without floating-point rounding and preserves incomplete states', () => {
    const category = {
      id: 'category-1', name: 'Hogar', sort_order: 10, assigned: '3', assignment_revision: 1,
      activity: { state: 'complete' as const, amount: '1' }, remaining: { state: 'complete' as const, amount: '-0.50' },
    }
    expect(usagePercent(category)).toBe(33.33)
    expect(categoryOverused(category)).toBe(true)
    expect(usagePercent({ ...category, activity: { state: 'incomplete' as const, known_subtotal: '1' } })).toBeNull()
  })
})
