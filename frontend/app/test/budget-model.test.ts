import { describe, expect, it } from 'vitest'
import { budgetRoute, currentPeriodInSantiago, describeBudgetValue, normalizeCategoryDraft, unresolvedDescriptions, type BudgetValue } from '../src/components/budget/budget-model'

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
})
