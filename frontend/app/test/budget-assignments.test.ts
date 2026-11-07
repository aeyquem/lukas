import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { ApiClientError } from '../src/shared/api/client'
import { useBudgetAssignments } from '../src/features/budget/composables/use-budget-assignments'
import type { BudgetSnapshot } from '../src/features/budget/api/contracts'
import { sendFinancialWrite } from '../src/shared/api/pending-commands'

vi.mock('../src/shared/api/pending-commands', () => ({ sendFinancialWrite: vi.fn() }))

let app: ReturnType<typeof createApp>

beforeEach(() => {
  vi.resetAllMocks()
  const pinia = createPinia()
  setActivePinia(pinia)
  app = createApp({})
  app.use(pinia)
  app.use(PiniaColada, { pinia })
})

function setup<T>(fn: () => T): T {
  return app.runWithContext(() => effectScope().run(fn)!)
}

const snapshot: BudgetSnapshot = {
  period: '2026-10', revision: '3', funding: { state: 'complete', amount: '100' },
  ready_to_assign: { state: 'complete', amount: '100' }, activity: { state: 'complete', amount: '0' },
  uncategorized_activity: { state: 'complete', amount: '0' }, complete: true,
  categories: [{ id: 'category-1', name: 'Hogar', sort_order: 10, assigned: '50', assignment_revision: 4, activity: { state: 'complete', amount: '0' }, remaining: { state: 'complete', amount: '50' } }],
}

it('keeps dirty assignment drafts and reviews revisions after a conflict', async () => {
  const error = new ApiClientError({ kind: 'conflict', status: 409, code: 'revision_conflict', message: 'conflict' })
  vi.mocked(sendFinancialWrite).mockRejectedValue(error)
  const period = ref('2026-10')
  const budget = ref<BudgetSnapshot | null>(snapshot)
  const pageError = ref('')
  const staleMessage = ref('')
  const success = ref('')
  const refresh = vi.fn(async () => {})
  const restorePending = vi.fn()
  const assignments = setup(() => useBudgetAssignments({
    period, budget, refresh, restorePending, errorMessage: (cause) => String(cause),
    onError: (value) => { pageError.value = value },
    onStale: (value) => { staleMessage.value = value },
    onSuccess: (value) => { success.value = value },
  }))

  assignments.assignDraft('category-1', '75')
  await assignments.save('category-1')

  expect(sendFinancialWrite).toHaveBeenCalledWith(expect.objectContaining({
    route: '/api/budgets/2026-10/assignments/category-1',
    body: '{"amount":"75","expected_period_revision":"3","expected_revision":4}',
  }))
  expect(assignments.drafts.value['2026-10:category-1']).toBe('75')
  expect(refresh).toHaveBeenCalledOnce()
  expect(staleMessage.value).toContain('El presupuesto cambió')
  expect(pageError.value).toBe('ApiClientError: conflict')
})
