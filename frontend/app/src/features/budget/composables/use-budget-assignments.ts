import { ref, type Ref } from 'vue'
import { ApiClientError } from '../../../shared/api/client'
import { useBudgetWriteMutation } from '../api/mutations'
import type { BudgetSnapshot } from '../api/contracts'
import { prepareAssignmentWrite } from '../budget-model'

interface UseBudgetAssignmentsOptions {
  period: Readonly<Ref<string>>
  budget: Readonly<Ref<BudgetSnapshot | null>>
  refresh: (period?: string) => Promise<void>
  restorePending: () => void
  errorMessage: (error: unknown, fallback: string) => string
  onError: (message: string) => void
  onStale: (message: string) => void
  onSuccess: (message: string) => void
}

export function useBudgetAssignments(options: UseBudgetAssignmentsOptions) {
  const drafts = ref<Record<string, string>>({})
  const reasonDrafts = ref<Record<string, string>>({})
  const saving = ref<string | null>(null)
  const write = useBudgetWriteMutation()

  function draftKey(categoryId: string): string { return `${options.period.value}:${categoryId}` }
  function assignDraft(categoryId: string, value: string): void {
    drafts.value = { ...drafts.value, [draftKey(categoryId)]: value }
  }
  function assignReason(categoryId: string, value: string): void {
    reasonDrafts.value = { ...reasonDrafts.value, [draftKey(categoryId)]: value }
  }

  async function save(categoryId: string): Promise<void> {
    const budget = options.budget.value
    if (!budget) return
    const requestedPeriod = options.period.value
    const key = draftKey(categoryId)
    const assignment = prepareAssignmentWrite(budget, categoryId, drafts.value[key], reasonDrafts.value[key] ?? '')
    if (!assignment) return
    if (assignment.error) {
      options.onError(assignment.error)
      return
    }
    saving.value = categoryId
    options.onError('')
    options.onStale('')
    options.onSuccess('')
    try {
      await write.mutateAsync({ method: 'PUT', route: `/api/budgets/${requestedPeriod}/assignments/${categoryId}`, body: assignment.body })
      const nextDrafts = { ...drafts.value }
      delete nextDrafts[key]
      drafts.value = nextDrafts
      const nextReasons = { ...reasonDrafts.value }
      delete nextReasons[key]
      reasonDrafts.value = nextReasons
      options.onSuccess(`Asignación guardada para ${assignment.categoryName}.`)
      options.restorePending()
      await options.refresh(requestedPeriod)
    } catch (error) {
      if (error instanceof ApiClientError && error.kind === 'conflict' && options.period.value === requestedPeriod) {
        const conflictMessage = `El presupuesto cambió. El borrador de ${assignment.categoryName} se conserva; actualiza los datos y vuelve a guardar.`
        await options.refresh(requestedPeriod)
        if (options.period.value === requestedPeriod) options.onStale(conflictMessage)
      }
      options.restorePending()
      if (options.period.value === requestedPeriod) options.onError(options.errorMessage(error, 'No fue posible guardar la asignación.'))
    } finally {
      saving.value = null
    }
  }

  return { drafts, reasonDrafts, saving, draftKey, assignDraft, assignReason, save }
}
