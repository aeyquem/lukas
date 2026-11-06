<script setup lang="ts">
import { ref, watch } from 'vue'
import { useAccountWriteMutation } from '../api/mutations'
import { useAccountForm } from '../composables/use-account-form'
import type { OpeningView } from '../api/contracts'
import { useAccountDetailQuery } from '../api/queries'
import AccountActivity from './account-activity.vue'
import OpeningCorrectionForm from './opening-correction-form.vue'
import { formatMoney } from '../../../shared/formatting/money'
import { formatBusinessDate } from '../../../shared/formatting/business-date'
const { fieldErrors, pageError, successMessage, fieldError, clearErrors, receiveError } = useAccountForm()
const mutation = useAccountWriteMutation()
const isSaving = mutation.isLoading
const props = defineProps<{
  accountId: string
  refreshToken: number
}>()
const emit = defineEmits<{
  close: []
  settled: []
}>()
const query = useAccountDetailQuery(() => props.accountId)
watch(() => props.refreshToken, () => {
  void query.refetch()
})
const account = query.data
const isLoading = query.isLoading
const editName = ref('')
const baselineRevision = ref<number | null>(null)
const openingCorrection = ref<{
  walletId: string
  opening: OpeningView
} | null>(null)
let correctionSession = 0
// Initial read initializes the draft once. Refetches cannot advance its baseline.
watch(account, value => {
  if (value && baselineRevision.value === null) {
    editName.value = value.name
    baselineRevision.value = value.revision
  }
}, { immediate: true })

function startOpeningCorrection(walletId: string, opening: OpeningView) {
  clearErrors()
  correctionSession++
  openingCorrection.value = { walletId, opening: { ...opening } }
}

async function submitNameEdit() {
  clearErrors()
  if (!editName.value.trim()) {
    fieldErrors.name = 'Escribe un nombre para la cuenta.'
    return
  }
  if (baselineRevision.value === null)
    return
  try {
    const outcome = await mutation.mutateAsync({ method: 'PUT', route: `/api/accounts/${props.accountId}`, payload: { name: editName.value.trim(), expected_revision: baselineRevision.value } })
    successMessage.value = `Nombre guardado (revisión ${outcome.receipt.revision}).`
    baselineRevision.value = outcome.receipt.revision
    if (outcome.readError !== undefined)
      pageError.value = 'El nombre quedó guardado, pero no se pudo actualizar la lectura. Usa “Actualizar”; no vuelvas a enviar el cambio.'
  }
  catch (error) {
    receiveError(error)
  }
  finally {
    emit('settled')
  }
}
async function reloadDraft() {
  await query.refetch()
  if (query.error.value)
    return
  if (account.value) {
    editName.value = account.value.name
    baselineRevision.value = account.value.revision
    clearErrors()
  }
}
</script>
<template>
<section class="editor-panel" aria-labelledby="edit-title">
  <div class="section-heading">
    <div>
      <p class="eyebrow">Cuenta · revisión {{ account?.revision }}</p>
      <h2 id="edit-title">{{ account?.name }}</h2>
    </div>
    <button type="button" class="text-button" @click="emit('close')">Cerrar</button>
  </div>
  <p v-if="pageError" class="notice error-notice" role="alert">{{ pageError }}</p>
  <p v-if="successMessage" class="notice success-notice" role="status">{{ successMessage }}</p>
  <form class="name-edit" @submit.prevent="submitNameEdit" novalidate>
    <div class="field">
      <label for="edit-account-name">Nombre de la cuenta</label>
      <input id="edit-account-name" v-model="editName" required :aria-invalid="Boolean(fieldError('name'))" :aria-describedby="fieldError('name') ? 'edit-account-name-error' : undefined" />
      <p v-if="fieldError('name')" id="edit-account-name-error" class="field-error">{{ fieldError('name') }}</p>
    </div>
    <p class="help-copy">El tipo y las monedas quedan fijos después de registrar los saldos iniciales.</p>
    <div class="form-actions">
      <button type="submit" class="primary-button" :disabled="isSaving">Guardar nombre</button>
    </div>
  </form>
  <button type="button" class="text-button" @click="reloadDraft">Revisar y recargar nombre</button>
  <p v-if="query.error.value" role="alert" class="notice error-notice">{{ query.error.value.message }}</p>
  <section class="wallet-readbacks" aria-labelledby="wallets-title">
    <div class="section-heading">
      <h3 id="wallets-title">Saldos autoritativos</h3>
      <button type="button" class="text-button" :disabled="isLoading" @click="query.refetch()">Actualizar</button>
    </div>
    <article v-for="wallet in account?.wallets ?? []" :key="wallet.id" class="wallet-readback">
      <div class="readback-heading">
        <h4 class="numeric">{{ wallet.currency }} · {{ formatMoney(wallet.balance, wallet.currency) }}</h4>
      </div>
      <p v-if="wallet.openings.length === 0" class="muted">No hay saldos iniciales registrados.</p>
      <ul class="opening-list">
        <li v-for="opening in wallet.openings" :key="opening.id" class="opening-row">
          <div>
            <strong class="numeric">{{ formatMoney(opening.amount, wallet.currency) }}</strong>
            <span class="opening-date numeric">{{ formatBusinessDate(opening.business_date) }}</span>
            <small>Revisión {{ opening.revision }}<template v-if="opening.liability_basis_provenance"> · Base {{ opening.liability_basis_provenance }}</template></small>
            <small v-if="opening.liability_clp_basis" class="numeric">Base CLP: {{ formatMoney(opening.liability_clp_basis, 'CLP') }}</small>
          </div>
          <button type="button" class="text-button" @click="startOpeningCorrection(wallet.id, opening)">Corregir saldo</button>
        </li>
      </ul>
    </article>
  </section>
  <AccountActivity :account-id="accountId" />
  <OpeningCorrectionForm v-if="openingCorrection && account" :key="openingCorrection.opening.id + ':' + correctionSession" :account="account" :wallet-id="openingCorrection.walletId" :opening="openingCorrection.opening" @close="openingCorrection = null" @settled="emit('settled')" />
</section>
</template>
<style scoped>
.section-heading,
.readback-heading,
.form-actions,
.opening-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

h2,
h3,
h4,
p {
  margin-top: 0;
}

h2 {
  margin-bottom: 0;
  font-size: 22px;
  letter-spacing: -0.035em;
}

h3 {
  margin-bottom: 12px;
  font-size: 16px;
}

h4 {
  margin-bottom: 0;
  font-size: 16px;
}

.help-copy,
.muted,
.opening-row small {
  color: var(--muted);
  line-height: 1.5;
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

.opening-list {
  display: grid;
  margin: 0;
  padding: 0;
  list-style: none;
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

input {
  width: 100%;
  min-height: 44px;
  padding: 10px 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--control-radius);
  color: var(--text);
  background: var(--surface-raised);
  font: inherit;
}

input:focus {
  border-color: var(--accent);
}

input[aria-invalid='true'] {
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

.text-button {
  border: 0;
  padding: 7px 4px;
  color: var(--accent);
  background: transparent;
  font: inherit;
  font-size: 12px;
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
}

.form-actions {
  justify-content: end;
  margin-top: 20px;
}

.name-edit {
  padding: 18px;
  border-radius: var(--control-radius);
  background: var(--surface-soft);
}

.wallet-readbacks {
  margin-top: 24px;
}

.wallet-readback {
  margin-top: 12px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--control-radius);
}

.readback-heading {
  margin-bottom: 12px;
}

.opening-list {
  gap: 10px;
}

.opening-row {
  align-items: start;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.opening-row > div {
  display: grid;
  gap: 4px;
}

.opening-date {
  color: var(--muted);
  font-size: 12px;
}

.opening-row small {
  font-size: 11px;
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
.editor-panel {
    padding: 18px;
  }
}

input[inputmode="decimal"],
input[type="date"] { font-family: var(--font-numeric); }
</style>
