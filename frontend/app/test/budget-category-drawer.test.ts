import { beforeEach, expect, it, vi } from 'vitest'
import { createApp, effectScope, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { useBudgetCategories } from '../src/features/budget/composables/use-budget-categories'
import type { ExpenseCategory } from '../src/features/budget/api/contracts'
import { sendFinancialWrite } from '../src/shared/api/pending-commands'
import { ApiClientError } from '../src/shared/api/client'
vi.mock('../src/shared/api/pending-commands', () => ({ sendFinancialWrite: vi.fn() }))
let app: ReturnType<typeof createApp>
beforeEach(() => {
  vi.resetAllMocks()
  const pinia = createPinia(); setActivePinia(pinia)
  app = createApp({}); app.use(pinia); app.use(PiniaColada, { pinia })
})
function setup() {
  const categories = ref<ExpenseCategory[]>([{ id: 'cat-1', slug: 'hogar', name: 'Hogar', sort_order: 10, archived: false, group_id: 'group-1', revision: 2, can_delete: true }])
  const refresh = vi.fn(async () => {})
  const restorePending = vi.fn()
  const workflow = app.runWithContext(() => effectScope().run(() => useBudgetCategories({ categories, refresh, restorePending, errorMessage: (error) => String(error) }))!)
  return { categories, refresh, restorePending, workflow }
}
it('preserves category drafts and captured revisions through readback and reopen, then explicitly reviews a stale version', async () => {
  const { categories, workflow } = setup()
  workflow.edit(categories.value[0])
  workflow.drafts.value['cat-1'].name = 'Mi hogar'
  categories.value = [{ ...categories.value[0], name: 'Otro nombre', revision: 3 }]
  workflow.edit(categories.value[0])
  vi.mocked(sendFinancialWrite).mockRejectedValue(new ApiClientError({ kind: 'conflict', status: 409, code: 'revision_conflict', message: 'stale' }))
  await expect(workflow.save('cat-1')).resolves.toBe(false)
  expect(JSON.parse(vi.mocked(sendFinancialWrite).mock.calls[0][0].body)).toMatchObject({ name: 'Mi hogar', expected_revision: 2, group_id: 'group-1' })
  expect(workflow.drafts.value['cat-1'].name).toBe('Mi hogar')
  expect(workflow.error.value).toContain('El borrador se conserva')
  workflow.review('cat-1')
  expect(workflow.drafts.value['cat-1'].expected_revision).toBe(3)
  expect(workflow.drafts.value['cat-1'].name).toBe('Mi hogar')
})
it('archives a referenced category through the API while protecting permanent deletion', async () => {
  const { categories, workflow } = setup()
  categories.value[0].can_delete = false
  await expect(workflow.remove('cat-1')).resolves.toBe(false)
  expect(sendFinancialWrite).not.toHaveBeenCalled()
  vi.mocked(sendFinancialWrite).mockResolvedValue({ resource_id: 'cat-1', revision: 3 })
  await expect(workflow.setArchived('cat-1', true)).resolves.toBe(true)
  expect(JSON.parse(vi.mocked(sendFinancialWrite).mock.calls[0][0].body)).toMatchObject({ archived: true, expected_revision: 2 })
})
it('deletes an eligible category with its expected revision and keeps confirmation distinct from refresh failure', async () => {
  const { refresh, workflow } = setup()
  vi.mocked(sendFinancialWrite).mockResolvedValue({ resource_id: 'cat-1', revision: 3 })
  refresh.mockRejectedValue(new Error('read failed'))
  await expect(workflow.remove('cat-1')).resolves.toBe(true)
  expect(sendFinancialWrite).toHaveBeenCalledExactlyOnceWith({ method: 'DELETE', route: '/api/categories/cat-1', body: '{"expected_revision":2}' })
  expect(workflow.notice.value).toContain('eliminada permanentemente')
  expect(workflow.error.value).toContain('El cambio está confirmado')
})
it('retains unknown-outcome draft until exact pending confirmation and clears it after receipt', async () => {
  const { categories, workflow, restorePending } = setup()
  workflow.edit(categories.value[0]); workflow.drafts.value['cat-1'].name = 'Cambio pendiente'
  vi.mocked(sendFinancialWrite).mockRejectedValue(new Error('unknown outcome'))
  await expect(workflow.save('cat-1')).resolves.toBe(false)
  expect(workflow.drafts.value['cat-1'].name).toBe('Cambio pendiente')
  expect(restorePending).toHaveBeenCalledOnce()
  await workflow.confirmPending('/api/categories/cat-1')
  expect(workflow.drafts.value['cat-1']).toBeUndefined()
  expect(workflow.notice.value).toContain('pendiente se confirmó')
})
