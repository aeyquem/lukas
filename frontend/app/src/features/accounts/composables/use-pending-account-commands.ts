import { ref, watch, type MaybeRefOrGetter, toValue } from 'vue'
import { listPendingAccountWrites } from '../api/write-flow'
import { dismissPendingWrite, PendingWriteStore, type PendingFinancialWrite } from '../../../shared/api/pending-commands'
import { useAccountRetryMutation } from '../api/mutations'
import { useAccountForm } from './use-account-form'
export function usePendingAccountCommands(refreshToken: MaybeRefOrGetter<number>) {
  const { pageError, successMessage, clearErrors, receiveError } = useAccountForm()
  const mutation = useAccountRetryMutation()
  const isSaving = mutation.isLoading
  const pendingAccountWrites = ref<PendingFinancialWrite[]>([])
  const pendingError = ref('')
  const dismissedPendingKeys = ref(new Set<string>())
  function loadPendingAccountWrites(): void {
    try {
      pendingAccountWrites.value = listPendingAccountWrites(new PendingWriteStore())
      pendingError.value = ''
    }
    catch (error) {
      pendingError.value = error instanceof Error
        ? error.message
        : 'No se pudieron recuperar los comandos pendientes.'
    }
  }
  function dismissAccountWrite(command: PendingFinancialWrite): void {
    try {
      dismissPendingWrite(command.idempotencyKey)
      dismissedPendingKeys.value.add(command.idempotencyKey)
    }
    catch (error) {
      pageError.value = error instanceof Error ? error.message : 'No se pudo ocultar el aviso.'
    }
  }
  function pendingWriteLabel(command: PendingFinancialWrite): string {
    if (command.route === '/api/accounts')
      return 'Crear cuenta'
    if (command.route.includes('/openings/'))
      return 'Corregir saldo inicial'
    return 'Editar nombre de cuenta'
  }
  watch(() => toValue(refreshToken), loadPendingAccountWrites, { immediate: true })
  async function retryAccountWrite(command: PendingFinancialWrite) {
    clearErrors()
    try {
      const outcome = await mutation.mutateAsync(command)
      successMessage.value = `La operación quedó confirmada (revisión ${outcome.receipt.revision}).`
      if (outcome.readError !== undefined)
        pageError.value = 'La operación quedó confirmada, pero no se pudo actualizar la lectura. Usa “Actualizar cuentas”; no vuelvas a enviar la operación.'
    }
    catch (error) {
      receiveError(error)
    }
    finally {
      loadPendingAccountWrites()
    }
  }
  return { pageError, successMessage, isSaving, pendingAccountWrites, pendingError, dismissedPendingKeys, retryAccountWrite, dismissAccountWrite, pendingWriteLabel }
}
