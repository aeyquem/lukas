import { reactive, shallowRef } from 'vue'
import { ApiClientError } from '../../../shared/api/client'
import { fieldMessage } from '../account-draft'
export { fieldMessage } from '../account-draft'
export function useAccountForm() {
  const fieldErrors = reactive<Record<string, string>>({})
  const pageError = shallowRef('')
  const successMessage = shallowRef('')
  const fieldError = (name: string) => fieldErrors[name]
  function clearErrors() {
    for (const key of Object.keys(fieldErrors)) delete fieldErrors[key]
    pageError.value = ''
    successMessage.value = ''
  }
  function receiveError(error: unknown) {
    if (error instanceof ApiClientError) {
      for (const item of error.fields)
        fieldErrors[item.field] = fieldMessage(item.code)
      pageError.value = error.outcomeUnknown
        ? 'No se pudo confirmar el resultado. Revisa las operaciones por confirmar.'
        : error.status === 409
          ? 'La cuenta cambió. Tu borrador se conserva. Actualiza la lectura y revisa el nombre o cancela y vuelve a abrir la corrección antes de guardar.'
          : error.fields.length ? '' : error.message
    }
    else
      pageError.value = error instanceof Error ? error.message : 'No fue posible guardar los cambios.'
  }
  return { fieldErrors, pageError, successMessage, fieldError, clearErrors, receiveError }
}
