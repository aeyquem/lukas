// Isolated UI fixture: API responses stay in memory and never reach a database.
import { createApp, h, ref } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import BudgetPlanningTable from '../../src/features/budget/components/budget-planning-table.vue'
import { useBudgetAssignments } from '../../src/features/budget/composables/use-budget-assignments'
import type { BudgetSnapshot, BudgetGroup, ExpenseCategory } from '../../src/features/budget/api/contracts'
import '../../src/styles.css'

export function mountBudgetTable() {
  const groups = ref<BudgetGroup[]>([
    { id: 'general', name: 'General', sort_order: 0, archived: false, revision: 1 },
    { id: 'empty', name: 'Grupo vacío', sort_order: 1, archived: false, revision: 1 },
  ])
  const categories = ref<ExpenseCategory[]>(['food', 'rent'].map((id, index) => ({ id, name: id === 'food' ? 'Comida' : 'Arriendo', slug: id, group_id: 'general', archived: false, sort_order: index, revision: 1, can_delete: true })))
  const complete = (amount = '0') => ({ state: 'complete' as const, amount })
  const period = ref('2026-11')
  const budget = ref<BudgetSnapshot>({
    period: period.value, revision: '1', closed: false, complete: true, funding: complete('1000'), ready_to_assign: complete('900'), activity: complete(), uncategorized_activity: complete(), archived_categories: [], groups: groups.value,
    available_to_budget_breakdown: Object.fromEntries(['prior_unassigned', 'income', 'openings', 'transfers', 'valuation', 'corrections', 'category_releases', 'category_rollover', 'assignments', 'funding_total'].map(key => [key, complete()])) as BudgetSnapshot['available_to_budget_breakdown'],
    categories: categories.value.map(c => ({ ...c, assigned: '50', assignment_revision: 1, activity: complete('10'), remaining: complete('40') })),
  })
  const writes: Array<{ route: string; body: Record<string, unknown> }> = []
  const pendingRoutes = ref<string[]>([])
  const rejectWrite = ref(false)
  const nativeFetch = window.fetch
  window.fetch = async (url, options) => {
    const route = String(url)
    if (route.endsWith('/bootstrap')) return Response.json({ installation_generation: '11111111-1111-4111-8111-111111111111' })
    if (!options?.method || options.method === 'GET') throw new Error(`Unexpected fixture request: ${route}`)
    const body = JSON.parse(String(options.body))
    writes.push({ route, body })
    if (rejectWrite.value) return Response.json({ error: { code: 'validation_error', message: 'Fixture rejected save' } }, { status: 422 })
    const id = route.split('/').pop()!
    if (route.includes('/assignments/')) {
      budget.value = { ...budget.value, revision: String(Number(budget.value.revision) + 1), categories: budget.value.categories.map(c => c.id === id ? { ...c, assigned: body.amount, assignment_revision: (c.assignment_revision ?? 0) + 1 } : c) }
    } else if (route.includes('/categories/')) {
      const category = categories.value.find(c => c.id === id)!
      Object.assign(category, body, { revision: category.revision + 1 })
      Object.assign(budget.value.categories.find(c => c.id === id)!, { group_id: body.group_id, sort_order: body.sort_order })
    }
    return Response.json({ resource_id: id, revision: 2 })
  }
  const refresh = async () => {}
  const root = document.createElement('div')
  document.body.replaceChildren(root)
  let assignments: ReturnType<typeof useBudgetAssignments>
  const app = createApp({ setup() {
    assignments = useBudgetAssignments({ period, budget, refresh, restorePending: () => {}, errorMessage: String, onError: () => {}, onStale: () => {}, onSuccess: () => {} })
    return () => h(BudgetPlanningTable, { budget: budget.value, groups: groups.value, categories: categories.value, assignments, disabled: assignments.saving.value !== null, pendingRoutes: pendingRoutes.value, refresh, restorePending: () => {} })
  } })
  const pinia = createPinia()
  app.use(pinia).use(PiniaColada, { pinia }).mount(root)
  return { budget, period, pendingRoutes, rejectWrite, writes, assignments: assignments!, unmount: () => { app.unmount(); window.fetch = nativeFetch } }
}
