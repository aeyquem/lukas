import { expect, it } from 'vitest'
import { budgetValueReasons, budgetValueText, closeCategoryLabel, groupedBudgetSections } from '../src/features/budget/budget-model'
import type { BudgetCategory, BudgetGroup, BudgetSnapshot } from '../src/features/budget/api/contracts'

const group = (id: string, sort_order: number, archived = false): BudgetGroup => ({ id, name: id, sort_order, archived, revision: 1 })
const category = (id: string, group_id: string, sort_order: number, archived = false): BudgetCategory => ({ id, name: id, group_id, sort_order, archived, assigned: '1', assignment_revision: 1, activity: { state: 'complete', amount: '0' }, remaining: { state: 'complete', amount: '1' } })

it('uses persisted group and category order and excludes archived planning items', () => {
  const groups = [group('second', 20), group('first', 10), group('archived-group', 0, true)]
  const categories = [category('later', 'first', 20), category('earlier', 'first', 10), category('hidden', 'first', 0, true), category('other', 'second', 0), category('in-archived', 'archived-group', 0)]
  const result = groupedBudgetSections(categories, groups)
  expect(result.map((item) => [item.group.id, item.categories.map((entry) => entry.id)])).toEqual([['first', ['earlier', 'later']], ['second', ['other']]])
  expect(categories[0].id).toBe('later')
  expect(groups[0].id).toBe('second')
})

it('keeps absent and incomplete component amounts distinct from literal rollover zero', () => {
  expect(budgetValueText(undefined)).toBe('No disponible')
  expect(budgetValueText({ state: 'complete' })).toBe('No disponible')
  expect(budgetValueText({ state: 'complete', amount: '0' })).toContain('0')
  expect(budgetValueText({ state: 'incomplete', known_subtotal: '0' })).toContain('incompleto')
  expect(budgetValueText({ state: 'incomplete' })).toBe('Incompleto')
  expect(budgetValueReasons({ state: 'unavailable', reasons: ['historical_funding_components_unavailable'] })).toEqual(['historical_funding_components_unavailable'])
  expect(budgetValueReasons({ state: 'incomplete', unresolved: [{ kind: 'missing_rate', amount: '3.50', currency: 'USD' }] })).toEqual(['missing_rate: 3.50 USD'])
})


it('identifies archived categories and their signed balance in close choices', () => {
  const archived = { ...category('retained', 'first', 0, true), name: 'Historical groceries', remaining: { state: 'complete' as const, amount: '-15.50' } }
  const snapshot = { categories: [], archived_categories: [archived] } as unknown as BudgetSnapshot
  expect(closeCategoryLabel(snapshot, 'retained')).toContain('Historical groceries (archivada)')
  expect(closeCategoryLabel(snapshot, 'retained')).toContain('-15,50')
  expect(closeCategoryLabel(snapshot, 'absent')).toBe('Categoría no disponible')
})
