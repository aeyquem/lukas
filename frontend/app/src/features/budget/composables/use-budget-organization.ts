import { ref, type Ref } from 'vue'
import type { BudgetGroup, ExpenseCategory } from '../api/contracts'
import { categoryWriteIntent, groupWriteIntent, useBudgetWriteMutation } from '../api/mutations'

interface OrganizationOptions {
  groups: Readonly<Ref<readonly BudgetGroup[]>>
  categories: Readonly<Ref<readonly ExpenseCategory[]>>
  refresh: () => Promise<void>
  restorePending: () => void
}

export function useBudgetOrganization(options: OrganizationOptions) {
  const editing = ref(false)
  const saving = ref(false)
  const error = ref('')
  const notice = ref('')
  const write = useBudgetWriteMutation()

  async function persist(action: () => Promise<void>): Promise<void> {
    if (saving.value) return
    saving.value = true; error.value = ''; notice.value = ''
    try { await action(); notice.value = 'Organización guardada.' }
    catch (cause) { error.value = cause instanceof Error ? cause.message : 'No fue posible guardar la organización. Se actualizarán los datos vigentes.' }
    finally {
      options.restorePending()
      try { await options.refresh() }
      catch (cause) { error.value = `No fue posible actualizar la organización: ${cause instanceof Error ? cause.message : 'error de lectura'}` }
      saving.value = false
    }
  }

  async function reorderGroups(orderedIds: readonly string[]): Promise<void> {
    if (!editing.value) return
    const groups = orderedIds.map((id) => options.groups.value.find((item) => item.id === id)).filter((item): item is BudgetGroup => !!item)
    await persist(async () => {
      for (let index = 0; index < groups.length; index++) {
        const item = groups[index]
        if (item.sort_order === index) continue
        await write.mutateAsync(groupWriteIntent(item.id, { name: item.name, archived: item.archived, sort_order: index, expected_revision: item.revision }))
      }
    })
  }

  async function reorderCategories(groupId: string, orderedIds: readonly string[]): Promise<void> {
    const categories = orderedIds.map((id) => options.categories.value.find((item) => item.id === id)).filter((item): item is ExpenseCategory => !!item)
    await persist(async () => {
      for (let index = 0; index < categories.length; index++) {
        const item = categories[index]
        if (item.sort_order === index && item.group_id === groupId) continue
        await write.mutateAsync(categoryWriteIntent(item.id, { name: item.name, archived: item.archived, group_id: groupId, sort_order: index, expected_revision: item.revision }))
      }
    })
  }

  return { editing, saving, error, notice, reorderGroups, reorderCategories }
}
