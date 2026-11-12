import { ref, shallowRef, type Ref } from 'vue'
import { ApiClientError } from '../../../shared/api/client'
import type { CategoryDraft, ExpenseCategory } from '../api/contracts'
import { categoryDeleteIntent, categoryWriteIntent, useBudgetWriteMutation } from '../api/mutations'
import { normalizeCategoryDraft } from '../budget-model'

interface UseBudgetCategoriesOptions {
  categories: Readonly<Ref<readonly ExpenseCategory[]>>
  refresh: () => Promise<void>
  restorePending: () => void
  errorMessage: (error: unknown, fallback: string) => string
}

export function useBudgetCategories(options: UseBudgetCategoriesOptions) {
  const drafts = ref<Record<string, CategoryDraft>>({})
  const newCategory = ref({ name: '', sort_order: '100', group_id: '' })
  const error = shallowRef('')
  const notice = shallowRef('')
  const saving = shallowRef<string | null>(null)
  const write = useBudgetWriteMutation()

  function edit(category: ExpenseCategory): void {
    // Opening details and query readback must not replace a dirty draft.
    if (drafts.value[category.id]) return
    drafts.value = {
      ...drafts.value,
      [category.id]: {
        name: category.name, sort_order: String(category.sort_order), archived: category.archived,
        group_id: category.group_id, expected_revision: category.revision,
      },
    }
    error.value = ''
  }

  function cancel(categoryId: string): void {
    const next = { ...drafts.value }
    delete next[categoryId]
    drafts.value = next
  }

  function review(categoryId: string): void {
    const category = options.categories.value.find((item) => item.id === categoryId)
    const draft = drafts.value[categoryId]
    if (category && draft) {
      drafts.value = { ...drafts.value, [categoryId]: { ...draft, expected_revision: category.revision } }
      error.value = ''
      notice.value = 'Se conserva el borrador y se usará la versión revisada al guardar.'
    }
  }

  function resetNew(): void {
    newCategory.value = {
      name: '', group_id: newCategory.value.group_id,
      sort_order: String(Math.min(32767, Math.max(0, ...options.categories.value.map((item) => item.sort_order)) + 10)),
    }
  }

  async function refreshConfirmed(): Promise<void> {
    try { await options.refresh() }
    catch (cause) {
      error.value = `El cambio está confirmado, pero no se pudo actualizar la lectura: ${options.errorMessage(cause, 'error de lectura')}`
    }
  }

  async function saveNew(): Promise<boolean> { return save(null) }

  async function setArchived(categoryId: string, archived: boolean): Promise<boolean> {
    const category = options.categories.value.find((item) => item.id === categoryId)
    if (!category) return false
    edit(category)
    const currentDraft = drafts.value[categoryId]
    drafts.value = { ...drafts.value, [categoryId]: { ...currentDraft, archived } }
    return save(categoryId)
  }

  async function save(categoryId: string | null): Promise<boolean> {
    if (saving.value !== null) return false
    const draft = categoryId ? drafts.value[categoryId] : newCategory.value
    const normalized = normalizeCategoryDraft(draft?.name ?? '', draft?.sort_order ?? '')
    if (!normalized || !draft?.group_id) {
      error.value = 'Escribe un nombre y selecciona un grupo activo.'
      return false
    }
    error.value = ''; notice.value = ''; saving.value = categoryId ?? 'category-create'
    try {
      await write.mutateAsync(categoryWriteIntent(categoryId, {
        ...normalized, group_id: draft.group_id,
        ...(categoryId ? {
          archived: drafts.value[categoryId]?.archived ?? false,
          expected_revision: drafts.value[categoryId]?.expected_revision,
        } : {}),
      }))
      if (categoryId) cancel(categoryId)
      else resetNew()
      notice.value = categoryId ? 'Categoría actualizada.' : 'Categoría añadida.'
      options.restorePending()
      await refreshConfirmed()
      return true
    } catch (cause) {
      error.value = options.errorMessage(cause, 'No fue posible guardar la categoría.')
      if (cause instanceof ApiClientError && cause.kind === 'conflict') {
        error.value += ' El borrador se conserva. Actualiza y revisa la versión antes de volver a guardar.'
      }
      options.restorePending()
      return false
    } finally { saving.value = null }
  }

  async function remove(categoryId: string): Promise<boolean> {
    if (saving.value !== null) return false
    const category = options.categories.value.find((item) => item.id === categoryId)
    if (!category?.can_delete) {
      error.value = 'La categoría conserva transacciones o historial presupuestario. Puedes archivarla.'
      return false
    }
    error.value = ''; notice.value = ''; saving.value = categoryId
    try {
      await write.mutateAsync(categoryDeleteIntent(categoryId, drafts.value[categoryId]?.expected_revision ?? category.revision))
      cancel(categoryId)
      notice.value = 'Categoría eliminada permanentemente.'
      options.restorePending()
      await refreshConfirmed()
      return true
    } catch (cause) {
      error.value = cause instanceof ApiClientError && cause.code === 'category_in_use'
        ? 'La categoría conserva transacciones o historial presupuestario. Puedes archivarla.'
        : options.errorMessage(cause, 'No fue posible eliminar la categoría.')
      options.restorePending()
      return false
    } finally { saving.value = null }
  }

  async function confirmPending(commandRoute: string): Promise<void> {
    const categoryId = commandRoute.match(/^\/api\/categories\/([^/]+)$/)?.[1]
    if (categoryId) cancel(categoryId)
    else resetNew()
    notice.value = 'El cambio de categoría pendiente se confirmó.'
    await refreshConfirmed()
  }

  return { drafts, newCategory, error, notice, saving, edit, cancel, review, save, saveNew, setArchived, remove, confirmPending }
}
