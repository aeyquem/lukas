import { ref, type Ref } from 'vue'
import { ApiClientError } from '../../../shared/api/client'
import { useBudgetWriteMutation } from '../api/mutations'
import type { BudgetSnapshot } from '../api/contracts'
import { prepareAssignmentWrite } from '../budget-model'
import type { PendingFinancialWrite } from '../../../shared/api/pending-commands'

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
  // A draft keeps the snapshot it was started against. Reads never silently
  // authorize overwriting a newer assignment.
  const baselines = ref<Record<string, BudgetSnapshot>>({})
  const saving = ref<string | null>(null)
  const write = useBudgetWriteMutation()

  function draftKey(categoryId: string): string { return `${options.period.value}:${categoryId}` }
  function assignDraft(categoryId: string, value: string): void {
    captureBaseline(categoryId)
    drafts.value = { ...drafts.value, [draftKey(categoryId)]: value }
  }
  function assignReason(categoryId: string, value: string): void {
    captureBaseline(categoryId)
    reasonDrafts.value = { ...reasonDrafts.value, [draftKey(categoryId)]: value }
  }

  function discardDraft(categoryId: string): void {
    const key = draftKey(categoryId)
    const nextDrafts = { ...drafts.value }
    const nextReasons = { ...reasonDrafts.value }
    const nextBaselines = { ...baselines.value }
    delete nextDrafts[key]
    delete nextReasons[key]
    delete nextBaselines[key]
    drafts.value = nextDrafts
    reasonDrafts.value = nextReasons
    baselines.value = nextBaselines
  }

  function captureBaseline(categoryId: string): void {
    const key = draftKey(categoryId)
    const current = options.budget.value
    if (current && !baselines.value[key]) baselines.value = { ...baselines.value, [key]: current }
  }

  function isStale(categoryId: string): boolean {
    const baseline = baselines.value[draftKey(categoryId)]
    const current = options.budget.value
    return !!baseline && !!current && (baseline.revision !== current.revision ||
      baseline.categories.find((item) => item.id === categoryId)?.assignment_revision !==
      current.categories.find((item) => item.id === categoryId)?.assignment_revision)
  }

  function reviewCurrent(categoryId: string): void {
    const current = options.budget.value
    if (current) baselines.value = { ...baselines.value, [draftKey(categoryId)]: current }
  }

  function confirmPending(command: PendingFinancialWrite): void {
    const match = command.route.match(/^\/api\/budgets\/(\d{4}-\d{2})\/assignments\/([^/]+)$/)
    if (!match) return
    const key = `${match[1]}:${match[2]}`
    const sent = JSON.parse(command.body) as { amount: string; reason?: string }
    // A receipt only resolves the draft actually sent, never newer local text.
    if (drafts.value[key] !== sent.amount || (reasonDrafts.value[key] ?? '').trim() !== (sent.reason ?? '')) return
    delete drafts.value[key]
    delete reasonDrafts.value[key]
    delete baselines.value[key]
  }

  async function save(categoryId: string): Promise<void> {
    if (saving.value !== null) return
    const budget = options.budget.value
    if (!budget) return
    const requestedPeriod = options.period.value
    const key = draftKey(categoryId)
    if (isStale(categoryId)) {
      options.onStale('El presupuesto cambió. Revisa los valores vigentes y confirma la revisión del borrador antes de guardar.')
      return
    }
    const assignment = prepareAssignmentWrite(baselines.value[key] ?? budget, categoryId, drafts.value[key], reasonDrafts.value[key] ?? '')
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
      const nextBaselines = { ...baselines.value }
      delete nextBaselines[key]
      baselines.value = nextBaselines
      const nextReasons = { ...reasonDrafts.value }
      delete nextReasons[key]
      reasonDrafts.value = nextReasons
      options.onSuccess(`Asignación guardada para ${assignment.categoryName}.`)
      options.restorePending()
      await options.refresh(requestedPeriod)
    } catch (error) {
      if (error instanceof ApiClientError && error.kind === 'conflict' && options.period.value === requestedPeriod) {
        const conflictMessage = `El presupuesto cambió. El borrador de ${assignment.categoryName} se conserva; revisa los valores vigentes y confirma la revisión del borrador antes de guardar.`
        await options.refresh(requestedPeriod)
        if (options.period.value === requestedPeriod) options.onStale(conflictMessage)
      }
      options.restorePending()
      if (options.period.value === requestedPeriod) options.onError(options.errorMessage(error, 'No fue posible guardar la asignación.'))
    } finally {
      saving.value = null
    }
  }

  return { drafts, reasonDrafts, saving, draftKey, assignDraft, assignReason, discardDraft, isStale, reviewCurrent, confirmPending, save }
}
