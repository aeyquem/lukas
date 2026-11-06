<script setup lang="ts">
import { reactive, watch } from 'vue'
import WalletEditor from './wallet-editor.vue'
import { useAccountWriteMutation } from '../api/mutations'
import { useAccountForm } from '../composables/use-account-form'
import { setDraftAccountType, type AccountType } from '../account-model'
import { accountTypes, validateAccountDraft, newDraft, isNegativeNonZero, type AccountDraft, type WalletDraft } from '../account-draft'
const { fieldErrors, pageError, successMessage, fieldError, clearErrors, receiveError } = useAccountForm()
const mutation = useAccountWriteMutation()
const isSaving = mutation.isLoading
const emit = defineEmits<{
  settled: []
}>()
const draft = reactive<AccountDraft>(newDraft())
const needsLiabilityBasis = (wallet: WalletDraft) => draft.budget_eligible && wallet.currency !== 'CLP' && isNegativeNonZero(wallet.opening_amount)
watch(() => draft.account_type, (type, previous) => {
  if (type === 'investment' && previous !== 'investment')
    draft.budget_eligible = false
})
function validateDraft(): boolean {
  clearErrors()
  Object.assign(fieldErrors, validateAccountDraft(draft))
  return Object.keys(fieldErrors).length === 0
}
function updateAccountType(event: Event): void {
  const accountType = (event.target as HTMLSelectElement).value as AccountType
  setDraftAccountType(draft, accountType)
}
async function submitCreate(): Promise<void> {
  if (!validateDraft())
    return
  let outcome
  try {
    const payload = {
      name: draft.name.trim(),
      account_type: draft.account_type,
      base_currency: draft.base_currency,
      budget_eligible: draft.budget_eligible,
      wallets: draft.wallets.map((wallet) => ({
        currency: wallet.currency,
        opening_amount: wallet.opening_amount,
        opening_date: wallet.opening_date,
        ...(needsLiabilityBasis(wallet) ? { liability_clp_basis: wallet.liability_clp_basis } : {}),
      })),
    }
    outcome = await mutation.mutateAsync({ method: 'POST', route: '/api/accounts', payload })
  }
  catch (error) {
    receiveError(error)
    emit('settled')
    return
  }
  emit('settled')
  const { receipt, value: account } = outcome
  Object.assign(draft, newDraft())
  successMessage.value = `La cuenta quedó creada (ID ${receipt.resource_id}).`
  if (account) {
    successMessage.value = 'La cuenta se creó y sus saldos quedaron registrados.'
  }
  else {
    pageError.value = 'La cuenta quedó creada, pero no se pudo actualizar su lectura. Usa “Actualizar cuentas”; no vuelvas a enviar la creación.'
  }
}
</script>
<template>
<section class="editor-panel" aria-labelledby="create-title">
  <div class="section-heading">
    <div>
      <p class="eyebrow">Nueva cuenta</p>
      <h2 id="create-title">Registra una cuenta</h2>
    </div>
  </div>
  <p v-if="pageError" class="notice error-notice" role="alert">{{ pageError }}</p>
  <p v-if="successMessage" class="notice success-notice" role="status">{{ successMessage }}</p>
  <form @submit.prevent="submitCreate" novalidate>
    <div class="form-grid">
      <div class="field full-field">
        <label for="account-name">Nombre</label>
        <input id="account-name" v-model="draft.name" autocomplete="off" required :aria-invalid="Boolean(fieldError('name'))" :aria-describedby="fieldError('name') ? 'account-name-error' : undefined" />
        <p v-if="fieldError('name')" id="account-name-error" class="field-error">{{ fieldError('name') }}</p>
      </div>
      <div class="field">
        <label for="account-type">Tipo</label>
        <select id="account-type" :value="draft.account_type" @change="updateAccountType">
          <option v-for="item in accountTypes" :key="item.value" :value="item.value">{{ item.label }}</option>
        </select>
        <p v-if="fieldError('account_type')" class="field-error">{{ fieldError('account_type') }}</p>
      </div>
      <div class="field">
        <label for="base-currency">Moneda base</label>
        <input id="base-currency" v-model="draft.base_currency" maxlength="3" autocapitalize="characters" spellcheck="false" required :aria-invalid="Boolean(fieldError('base_currency'))" :aria-describedby="fieldError('base_currency') ? 'base-currency-error' : undefined" @input="draft.base_currency = draft.base_currency.toUpperCase()" />
        <p v-if="fieldError('base_currency')" id="base-currency-error" class="field-error">{{ fieldError('base_currency') }}</p>
      </div>
      <label class="eligibility-field">
        <input v-model="draft.budget_eligible" type="checkbox" />
        <span>
          <strong>Participa en presupuesto</strong>
          <small>Las inversiones empiezan desactivadas; puedes habilitarlas cuando corresponda.</small>
        </span>
      </label>
    </div>
    <WalletEditor v-model="draft.wallets" :account-type="draft.account_type" :budget-eligible="draft.budget_eligible" :field-errors="fieldErrors" />
    <div class="form-actions">
      <button type="submit" class="primary-button" :disabled="isSaving">{{ isSaving ? 'Guardando…' : 'Guardar cuenta' }}</button>
    </div>
  </form>
</section>
</template>
<style scoped>
.section-heading,
.form-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

h2,
p {
  margin-top: 0;
}

h2 {
  margin-bottom: 0;
  font-size: 22px;
  letter-spacing: -0.035em;
}

.eyebrow {
  margin-bottom: 8px;
}

.editor-panel {
  border: 1px solid var(--border);
  border-radius: var(--island-radius);
  background: var(--surface);
  box-shadow: var(--shadow-island);
}

.editor-panel {
  min-width: 0;
  padding: var(--island-padding);
}

.section-heading {
  margin-bottom: 18px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.full-field {
  grid-column: 1 / -1;
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
select {
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
select:focus {
  border-color: var(--accent);
}

input[aria-invalid='true'] {
  border-color: var(--danger);
}

.eligibility-field {
  display: flex;
  grid-column: 1 / -1;
  align-items: start;
  gap: 12px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--control-radius);
  cursor: pointer;
}

.eligibility-field input {
  width: 18px;
  min-height: 18px;
  margin: 2px 0 0;
  accent-color: var(--accent);
}

.eligibility-field span {
  display: grid;
  gap: 3px;
}

.eligibility-field strong {
  font-size: 13px;
}

.eligibility-field small {
  color: var(--muted);
  line-height: 1.4;
}

.field-error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
  line-height: 1.4;
}

.primary-button {
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

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.form-actions {
  justify-content: end;
  margin-top: 20px;
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

@media (max-width: 760px) {
.form-grid {
    grid-template-columns: 1fr;
  }

.full-field,
.eligibility-field {
    grid-column: auto;
  }

.editor-panel {
    padding: 18px;
  }
}

input[inputmode="decimal"],
input[type="date"] { font-family: var(--font-numeric); }
</style>
