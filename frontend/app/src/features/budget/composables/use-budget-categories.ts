import { ref, type Ref } from 'vue'
import { ApiClientError } from '../../../shared/api/client'
import type { CategoryDraft, ExpenseCategory } from '../api/contracts'
import { useBudgetWriteMutation } from '../api/mutations'
import { normalizeCategoryDraft } from '../budget-model'

interface UseBudgetCategoriesOptions {
  categories: Readonly<Ref<readonly ExpenseCategory[]>>
  refresh: () => Promise<void>
  restorePending: () => void
  errorMessage: (error: unknown, fallback: string) => string
}

export function useBudgetCategories(options: UseBudgetCategoriesOptions) {
  const drafts = ref<Record<string, CategoryDraft>>({})
  const newCategory = ref({ name: '', sort_order: '100' })
  const error = ref('')
  const notice = ref('')
  const saving = ref<string | null>(null)
  const write = useBudgetWriteMutation()

  function edit(category: ExpenseCategory): void {
    drafts.value = {
      ...drafts.value,
      [category.id]: { name: category.name, sort_order: String(category.sort_order), archived: category.archived },
    }
  }

  function cancel(categoryId: string): void {
    const next = { ...drafts.value }
    delete next[categoryId]
    drafts.value = next
  }

  async function saveNew(): Promise<void> { await save(null) }

  async function setArchived(categoryId: string, archived: boolean): Promise<void> {
    const category = options.categories.value.find((item) => item.id === categoryId)
    if (!category) return
    if (!drafts.value[categoryId]) edit(category)
    const currentDraft = drafts.value[categoryId]
    if (!currentDraft) return
    drafts.value = { ...drafts.value, [categoryId]: { ...currentDraft, archived } }
    await save(categoryId)
  }

  async function save(categoryId: string | null): Promise<void> {
    const draft = categoryId ? drafts.value[categoryId] : newCategory.value
    const normalized = normalizeCategoryDraft(draft?.name ?? '', draft?.sort_order ?? '')
    if (!normalized) {
      error.value = 'Escribe un nombre y un orden entero entre 0 y 32767.'
      return
    }
    error.value = ''
    notice.value = ''
    saving.value = categoryId ?? 'category-create'
    const body = JSON.stringify({
      ...normalized,
      ...(categoryId ? { archived: drafts.value[categoryId]?.archived ?? false } : {}),
    })
    try {
      await write.mutateAsync({
        method: categoryId ? 'PUT' : 'POST',
        route: categoryId ? `/api/categories/${categoryId}` : '/api/categories',
        body,
      })
      if (categoryId) cancel(categoryId)
      else newCategory.value = {
        name: '',
        sort_order: String(Math.max(0, ...options.categories.value.map((item) => item.sort_order)) + 10),
      }
      notice.value = categoryId ? 'Categoría actualizada.' : 'Categoría añadida.'
      options.restorePending()
      await options.refresh()
    } catch (cause) {
      error.value = cause instanceof ApiClientError && cause.code === 'category_in_use'
        ? 'No se puede archivar: la categoría ya tiene actividad o asignaciones.'
        : options.errorMessage(cause, 'No fue posible guardar la categoría.')
      options.restorePending()
    } finally {
      saving.value = null
    }
  }

  async function confirmPending(commandRoute: string): Promise<void> {
    await options.refresh()
    const categoryId = commandRoute.match(/^\/api\/categories\/([^/]+)$/)?.[1]
    if (categoryId) cancel(categoryId)
    else newCategory.value = {
      name: '',
      sort_order: String(Math.max(0, ...options.categories.value.map((item) => item.sort_order)) + 10),
    }
    notice.value = 'El cambio de categoría pendiente se confirmó.'
  }

  return { drafts, newCategory, error, notice, saving, edit, cancel, save, saveNew, setArchived, confirmPending }
}
