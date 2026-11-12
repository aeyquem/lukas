import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { useBudgetOrganization } from '../src/features/budget/composables/use-budget-organization'
import { sendFinancialWrite } from '../src/shared/api/pending-commands'
import type { BudgetGroup, ExpenseCategory } from '../src/features/budget/api/contracts'

vi.mock('../src/shared/api/pending-commands', () => ({ sendFinancialWrite: vi.fn() }))
let app: ReturnType<typeof createApp>
beforeEach(() => {
  vi.resetAllMocks()
  const pinia = createPinia(); setActivePinia(pinia)
  app = createApp({}); app.use(pinia); app.use(PiniaColada, { pinia })
})
function harness() {
  const groups = ref<BudgetGroup[]>([{ id: 'general', name: 'General', sort_order: 0, archived: false, revision: 2 }, { id: 'savings', name: 'Savings', sort_order: 1, archived: false, revision: 3 }])
  const categories = ref<ExpenseCategory[]>([{ id: 'food', name: 'Food', slug: 'food', group_id: 'general', sort_order: 0, archived: false, revision: 5, can_delete: false }])
  const refresh = vi.fn(async () => {})
  const restorePending = vi.fn()
  const workflow = app.runWithContext(() => effectScope().run(() => useBudgetOrganization({ groups, categories, refresh, restorePending }))!)
  return { groups, categories, refresh, restorePending, workflow }
}
it('keeps group arrangement locked outside explicit edit mode', async () => {
  const { workflow, refresh } = harness()
  await workflow.reorderGroups(['savings', 'general'])
  expect(sendFinancialWrite).not.toHaveBeenCalled()
  expect(refresh).not.toHaveBeenCalled()
})
it('persists group order with original revisions and refreshes authoritative state', async () => {
  vi.mocked(sendFinancialWrite).mockResolvedValue({ resource_id: 'group', revision: 4 })
  const { workflow, groups, refresh } = harness()
  workflow.editing.value = true
  await workflow.reorderGroups(['savings', 'general'])
  expect(sendFinancialWrite).toHaveBeenNthCalledWith(1, { method: 'PUT', route: '/api/budget-groups/savings', body: '{"name":"Savings","archived":false,"sort_order":0,"expected_revision":3}' })
  expect(sendFinancialWrite).toHaveBeenNthCalledWith(2, { method: 'PUT', route: '/api/budget-groups/general', body: '{"name":"General","archived":false,"sort_order":1,"expected_revision":2}' })
  expect(refresh).toHaveBeenCalledOnce()
  expect(groups.value[0].id).toBe('general')
})
it('persists cross-group category membership and retains authoritative values until readback', async () => {
  vi.mocked(sendFinancialWrite).mockResolvedValue({ resource_id: 'food', revision: 6 })
  const { workflow, categories, refresh } = harness()
  expect(workflow.editing.value).toBe(false)
  await workflow.reorderCategories('savings', ['food'])
  expect(sendFinancialWrite).toHaveBeenCalledWith({ method: 'PUT', route: '/api/categories/food', body: '{"name":"Food","archived":false,"group_id":"savings","sort_order":0,"expected_revision":5}' })
  expect(categories.value[0].group_id).toBe('general')
  expect(refresh).toHaveBeenCalledOnce()
})
it('stops a partial reorder on write failure and reads back instead of guessing the outcome', async () => {
  vi.mocked(sendFinancialWrite).mockResolvedValueOnce({ resource_id: 'savings', revision: 4 }).mockRejectedValueOnce(new Error('revision conflict'))
  const { workflow, refresh, restorePending } = harness()
  workflow.editing.value = true
  await workflow.reorderGroups(['savings', 'general'])
  expect(sendFinancialWrite).toHaveBeenCalledTimes(2)
  expect(workflow.error.value).toBe('revision conflict')
  expect(workflow.notice.value).toBe('')
  expect(refresh).toHaveBeenCalledOnce()
  expect(restorePending).toHaveBeenCalledOnce()
  expect(workflow.saving.value).toBe(false)
})
