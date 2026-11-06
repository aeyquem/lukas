<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAccountWriteMutation } from '../api/mutations'
import { useAccountForm } from '../composables/use-account-form'
import type { AccountView, OpeningView } from '../api/contracts'
import { isValidMoney, isNegativeNonZero, signOf } from '../account-draft'
const { fieldErrors, pageError, successMessage, fieldError, clearErrors, receiveError } = useAccountForm()
const mutation = useAccountWriteMutation()
const isSaving = mutation.isLoading
const props = defineProps<{
  account: AccountView
  walletId: string
  opening: OpeningView
}>()
const emit = defineEmits<{
  close: []
  settled: []
}>()
const baseline = { ...props.opening }
const openingCorrection = ref({ walletId: props.walletId, opening: baseline })
const selectedAccount = computed(() => props.account)
const correctionAmount = ref(baseline.amount)
const correctionBasis = ref(baseline.liability_clp_basis ?? '')
const correctionReason = ref('')
const confirmed = ref(false)
const needsCorrectionBasis = computed(() => {
  const account = selectedAccount.value
  const correction = openingCorrection.value
  if (!account || !correction)
    return false
  const wallet = account.wallets.find((item) => item.id === correction.walletId)
  return Boolean(wallet &&
    account.budget_eligible &&
    wallet.currency !== 'CLP' &&
    (correction.opening.liability_clp_basis !== null || isNegativeNonZero(correctionAmount.value)))
})
async function submitOpeningCorrection(): Promise<void> {
  if (confirmed.value || isSaving.value) return
  const account = selectedAccount.value
  const correction = openingCorrection.value
  clearErrors()
  if (!account || !correction)
    return
  if (!isValidMoney(correctionAmount.value)) {
    fieldErrors.amount = 'Escribe un decimal válido con hasta 28 decimales y dentro del rango permitido.'
  }
  if (!correctionReason.value.trim())
    fieldErrors.reason = 'Explica brevemente el motivo del cambio.'
  const wallet = account.wallets.find((item) => item.id === correction.walletId)
  const basisRequired = Boolean(wallet &&
    account.budget_eligible &&
    wallet.currency !== 'CLP' &&
    (correction.opening.liability_clp_basis !== null || isNegativeNonZero(correctionAmount.value)))
  if (basisRequired && !isValidMoney(correctionBasis.value)) {
    fieldErrors.liability_clp_basis = 'Indica la base CLP firmada para esta deuda.'
  }
  if (basisRequired && isValidMoney(correctionBasis.value) && signOf(correctionBasis.value) !== signOf(correctionAmount.value)) {
    fieldErrors.liability_clp_basis = 'La base CLP debe tener el mismo signo que el saldo corregido.'
  }
  if (Object.keys(fieldErrors).length > 0)
    return
  let outcome
  try {
    const body = {
      amount: correctionAmount.value,
      expected_revision: correction.opening.revision,
      reason: correctionReason.value.trim(),
      ...(basisRequired
        ? {
          liability_clp_basis: correctionBasis.value,
          ...(correction.opening.liability_basis_revision === null
            ? {}
            : { expected_basis_revision: correction.opening.liability_basis_revision }),
        }
        : {}),
    }
    outcome = await mutation.mutateAsync({ method: 'PUT', route: `/api/accounts/${account.id}/openings/${correction.opening.id}`, payload: body })
  }
  catch (error) {
    receiveError(error)
    emit('settled')
    return
  }
  const { receipt, readError } = outcome
  confirmed.value = true
  successMessage.value = `Corrección guardada (revisión ${receipt.revision}).`
  if (readError !== undefined) {
    pageError.value = 'La corrección quedó guardada, pero no se pudo actualizar la lectura. Usa “Actualizar cuentas”; no vuelvas a enviar la corrección.'
  }
  emit('settled')
}
</script>
<template>
<form class="correction-panel" @submit.prevent="submitOpeningCorrection" novalidate>
  <p v-if="pageError" class="notice error-notice" role="alert">{{ pageError }}</p>
  <p v-if="successMessage" class="notice success-notice" role="status">{{ successMessage }}</p>
  <h3>Corregir saldo inicial</h3>
  <p class="help-copy">La corrección conserva el dato anterior y su revisión para auditoría.</p>
  <div class="field">
    <label for="correction-amount">Monto correcto</label>
    <input id="correction-amount" v-model="correctionAmount" inputmode="decimal" required :aria-invalid="Boolean(fieldError('amount'))" :aria-describedby="fieldError('amount') ? 'correction-amount-error' : undefined" />
    <p v-if="fieldError('amount')" id="correction-amount-error" class="field-error">{{ fieldError('amount') }}</p>
  </div>
  <div v-if="needsCorrectionBasis" class="field">
    <label for="correction-basis">Base CLP firmada</label>
    <input id="correction-basis" v-model="correctionBasis" inputmode="decimal" required :aria-invalid="Boolean(fieldError('liability_clp_basis'))" :aria-describedby="fieldError('liability_clp_basis') ? 'correction-basis-error' : undefined" />
    <p class="help-copy">La deuda extranjera necesita su propia base de financiamiento CLP.</p>
    <p v-if="fieldError('liability_clp_basis')" id="correction-basis-error" class="field-error">{{ fieldError('liability_clp_basis') }}</p>
  </div>
  <div class="field">
    <label for="correction-reason">Motivo</label>
    <textarea id="correction-reason" v-model="correctionReason" rows="3" required :aria-invalid="Boolean(fieldError('reason'))" :aria-describedby="fieldError('reason') ? 'correction-reason-error' : undefined"></textarea>
    <p v-if="fieldError('reason')" id="correction-reason-error" class="field-error">{{ fieldError('reason') }}</p>
  </div>
  <div class="form-actions">
    <button type="button" class="secondary-button" @click="emit('close')">Cancelar</button>
    <button type="submit" class="primary-button" :disabled="isSaving || confirmed">Guardar corrección</button>
  </div>
</form>
</template>
<style scoped>
.form-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

h3,
p {
  margin-top: 0;
}

h3 {
  margin-bottom: 12px;
  font-size: 16px;
}

.help-copy {
  color: var(--muted);
  line-height: 1.5;
}

.field {
  display: grid;
  align-content: start;
  gap: 7px;
  margin-bottom: 16px;
}

.field label {
  font-size: 13px;
  font-weight: 700;
}

input,
textarea {
  width: 100%;
  min-height: 44px;
  padding: 10px 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--control-radius);
  color: var(--text);
  background: var(--surface-raised);
  font: inherit;
}

input:focus,
textarea:focus {
  border-color: var(--accent);
}

input[aria-invalid='true'],
textarea[aria-invalid='true'] {
  border-color: var(--danger);
}

.help-copy {
  margin-bottom: 0;
  font-size: 12px;
}

.field-error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
  line-height: 1.4;
}

.primary-button,
.secondary-button {
  display: inline-flex;
  min-height: 44px;
  align-items: center;
  justify-content: center;
  padding: 10px 16px;
  border: 1px solid transparent;
  border-radius: var(--control-radius);
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.primary-button {
  color: var(--on-accent);
  background: var(--accent);
}

.primary-button:hover:not(:disabled) {
  background: var(--accent-hover);
}

.secondary-button {
  border-color: var(--border-strong);
  color: var(--accent);
  background: var(--surface-soft);
}

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.form-actions {
  justify-content: end;
  margin-top: 20px;
}

.correction-panel {
  margin-top: 22px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--control-radius);
  background: var(--surface-soft);
}

.notice {
  margin: 0 0 20px;
  padding: 12px 14px;
  border-radius: var(--control-radius);
  font-size: 14px;
}

.error-notice {
  color: var(--danger);
  background: var(--danger-bg);
}

.success-notice {
  color: var(--accent-hover);
  background: var(--success-bg);
}

input[inputmode="decimal"],
input[type="date"] { font-family: var(--font-numeric); }
</style>
