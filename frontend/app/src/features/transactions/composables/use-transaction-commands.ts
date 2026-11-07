import { computed, ref } from 'vue'
import { ApiClientError } from '../../../shared/api/client'
import {
  PendingWriteStore,
  dismissPendingWrite,
  type PendingFinancialWrite,
  type FinancialWriteIntent,
} from '../../../shared/api/pending-commands'
import { useTransactionRetryMutation, useTransactionWriteMutation } from '../api/mutations'
import type { TransactionMutationReceipt } from '../api/mutations'
import { transactionFieldMessage } from '../transaction-field-message'

function messageFor(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}

function hasPendingCommandKey(error: unknown): boolean {
  return typeof error === 'object' && error !== null && 'pendingCommandKey' in error
}

export function useTransactionCommands() {
  const writeMutation = useTransactionWriteMutation()
  const retryMutation = useTransactionRetryMutation()
  const pendingCommands = ref<PendingFinancialWrite[]>([])
  const dismissedPendingKeys = ref(new Set<string>())
  const serverErrors = ref<Record<string, string>>({})
  const pageError = ref('')
  const successMessage = ref('')
  const isSaving = ref(false)
  const visiblePendingCommands = computed(() => pendingCommands.value.filter(
    (command) => !dismissedPendingKeys.value.has(command.idempotencyKey),
  ))
  let pendingStore: PendingWriteStore | undefined

  function getPendingStore(): PendingWriteStore {
    pendingStore ??= new PendingWriteStore()
    return pendingStore
  }

  function restorePendingCommands(): void {
    try {
      pendingCommands.value = getPendingStore()
        .list()
        .filter((command) => command.route.startsWith('/api/transactions'))
    } catch (error) {
      pageError.value = messageFor(error, 'No fue posible leer los comandos guardados.')
    }
  }

  function dismissCommand(command: PendingFinancialWrite): void {
    try {
      dismissPendingWrite(command.idempotencyKey, getPendingStore())
    } catch (error) {
      pageError.value = messageFor(error, 'No fue posible ocultar el aviso.')
    }
    dismissedPendingKeys.value = new Set([...dismissedPendingKeys.value, command.idempotencyKey])
  }

  function receiveError(error: unknown, fallback: string): void {
    if (error instanceof ApiClientError && error.fields.length > 0) {
      serverErrors.value = Object.fromEntries(
        error.fields.map((field) => [field.field, transactionFieldMessage(field.code)]),
      )
    }
    pageError.value = messageFor(error, fallback)
  }

  async function afterConfirmedWrite(
    receipt: TransactionMutationReceipt,
    refreshTransactions: () => Promise<void>,
    afterConfirm: () => void,
    successPrefix: string,
    refreshFailure: string,
  ): Promise<boolean> {
    afterConfirm()
    successMessage.value = `${successPrefix} (revisión ${receipt.revision}).`
    restorePendingCommands()
    try {
      await refreshTransactions()
    } catch {
      pageError.value = refreshFailure
    }
    return true
  }

  async function submitWrite(
    command: FinancialWriteIntent,
    refreshTransactions: () => Promise<void>,
    afterConfirm: () => void,
  ): Promise<boolean> {
    pageError.value = ''
    successMessage.value = ''
    serverErrors.value = {}
    isSaving.value = true
    try {
      const receipt = await writeMutation.mutateAsync(command)
      return await afterConfirmedWrite(
        receipt,
        refreshTransactions,
        afterConfirm,
        'Movimiento guardado',
        'El movimiento se guardó, pero no se pudo actualizar la lista. Intenta recargarla.',
      )
    } catch (error) {
      receiveError(error, 'No fue posible guardar el movimiento.')
      if (hasPendingCommandKey(error)) restorePendingCommands()
      return false
    } finally {
      isSaving.value = false
    }
  }

  async function retryCommand(
    command: PendingFinancialWrite,
    refreshTransactions: () => Promise<void>,
    afterConfirm: () => void,
  ): Promise<boolean> {
    pageError.value = ''
    successMessage.value = ''
    isSaving.value = true
    try {
      const receipt = await retryMutation.mutateAsync(command)
      return await afterConfirmedWrite(
        receipt,
        refreshTransactions,
        afterConfirm,
        'El comando guardado se confirmó',
        'El comando se confirmó, pero no se pudo actualizar la lista. Intenta recargarla.',
      )
    } catch (error) {
      receiveError(error, 'No se pudo confirmar el comando pendiente.')
      restorePendingCommands()
      return false
    } finally {
      isSaving.value = false
    }
  }

  return {
    pendingCommands,
    visiblePendingCommands,
    serverErrors,
    pageError,
    successMessage,
    isSaving,
    restorePendingCommands,
    dismissCommand,
    submitWrite,
    retryCommand,
  }
}
