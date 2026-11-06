<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import {
  buildTransactionRequest,
  createTransactionDraft,
  draftFromTransaction,
  requiresOpeningTreatment,
  type AccountView,
  type TransactionCategory,
  type TransactionDraft,
  type TransactionView,
} from './transaction-model'
import OpeningTreatmentField from './OpeningTreatmentField.vue'

const props = defineProps<{
  accounts: AccountView[]
  categories: TransactionCategory[]
  expenses: TransactionView[]
  transaction: TransactionView | null
  saving: boolean
  serverErrors: Record<string, string>
  today: string
}>()

const emit = defineEmits<{
  submit: [command: { method: 'POST' | 'PUT'; route: string; body: string }]
  cancel: []
}>()

const moneyPattern = /^(0|[1-9][0-9]*)(?:\.[0-9]+)?$/
const draft = reactive<TransactionDraft>(createTransactionDraft(props.today))
const localErrors = reactive<Record<string, string>>({})
const isHydrating = ref(false)
const isEditing = computed(() => props.transaction !== null)
const allWallets = computed(() =>
  props.accounts.flatMap((account) =>
    account.wallets.map((wallet) => ({ ...wallet, accountName: account.name })),
  ),
)
const sourceExpenses = computed(() => props.expenses.filter((item) => item.kind === 'expense'))
const selectedWallet = computed(() => allWallets.value.find((wallet) => wallet.id === draft.walletId))
const selectedSourceWallet = computed(() =>
  allWallets.value.find((wallet) => wallet.id === draft.sourceWalletId),
)
const selectedDestinationWallet = computed(() =>
  allWallets.value.find((wallet) => wallet.id === draft.destinationWalletId),
)
const needsWalletOpeningChoice = computed(() =>
  requiresOpeningTreatment(draft.businessDate, selectedWallet.value),
)
const needsSourceOpeningChoice = computed(() =>
  requiresOpeningTreatment(draft.businessDate, selectedSourceWallet.value),
)
const needsDestinationOpeningChoice = computed(() =>
  requiresOpeningTreatment(draft.businessDate, selectedDestinationWallet.value),
)
const transferIsCrossCurrency = computed(
  () =>
    selectedSourceWallet.value !== undefined &&
    selectedDestinationWallet.value !== undefined &&
    selectedSourceWallet.value.currency !== selectedDestinationWallet.value.currency,
)
const confirmedClpIsRelevant = computed(
  () =>
    draft.kind === 'expense' &&
    (selectedWallet.value?.currency !== undefined && selectedWallet.value.currency !== 'CLP' ||
      props.transaction?.confirmed_clp_amount !== null && props.transaction?.confirmed_clp_amount !== undefined),
)
const showCategory = computed(() =>
  draft.kind === 'income' || draft.kind === 'expense' || (draft.kind === 'refund' && !draft.sourceExpenseId),
)

watch(
  () => props.transaction,
  (transaction) => {
    isHydrating.value = true
    Object.assign(
      draft,
      transaction ? draftFromTransaction(transaction, props.accounts) : createTransactionDraft(props.today),
    )
    isHydrating.value = false
    clearLocalErrors()
  },
  { immediate: true },
)

watch(
  () => props.accounts,
  (currentAccounts) => {
    if (props.transaction) {
      isHydrating.value = true
      Object.assign(draft, draftFromTransaction(props.transaction, currentAccounts))
      isHydrating.value = false
    }
  },
)

watch(
  () => draft.businessDate,
  (date, previous) => {
    if (!isHydrating.value && date !== previous) {
      // Changing chronology invalidates any prior choice; require a fresh
      // confirmation if the corrected date is before or on an opening.
      draft.openingTreatment = ''
      draft.sourceOpeningTreatment = ''
      draft.destinationOpeningTreatment = ''
    }
  },
  { flush: 'sync' },
)

watch(
  () => draft.walletId,
  () => {
    if (isHydrating.value) return
    draft.openingTreatment = ''
  },
  { flush: 'sync' },
)

watch(
  () => draft.sourceWalletId,
  () => {
    if (isHydrating.value) return
    draft.sourceOpeningTreatment = ''
  },
  { flush: 'sync' },
)

watch(
  () => draft.destinationWalletId,
  () => {
    if (isHydrating.value) return
    draft.destinationOpeningTreatment = ''
  },
  { flush: 'sync' },
)

function clearLocalErrors(): void {
  for (const key of Object.keys(localErrors)) delete localErrors[key]
}

function messageFor(field: string): string | undefined {
  return localErrors[field] ?? props.serverErrors[field]
}
function errorId(field: string): string { return `transaction-${field.replaceAll('_', '-')}-error` }

function validate(): boolean {
  clearLocalErrors()
  if (!draft.businessDate) localErrors.business_date = 'Selecciona la fecha real del movimiento.'

  if (draft.kind === 'transfer') {
    if (!draft.sourceWalletId) localErrors.source_wallet_id = 'Selecciona la billetera de origen.'
    if (!moneyPattern.test(draft.sourceAmount) || /^0(?:\.0+)?$/.test(draft.sourceAmount)) {
      localErrors.source_amount = 'Escribe un monto positivo sin separadores.'
    }
    if (!draft.destinationWalletId) localErrors.destination_wallet_id = 'Selecciona la billetera de destino.'
    if (!moneyPattern.test(draft.destinationAmount) || /^0(?:\.0+)?$/.test(draft.destinationAmount)) {
      localErrors.destination_amount = 'Escribe un monto positivo sin separadores.'
    }
    if (draft.sourceWalletId && draft.sourceWalletId === draft.destinationWalletId) {
      localErrors.destination_wallet_id = 'Elige una billetera distinta a la de origen.'
    }
    if (needsSourceOpeningChoice.value && !draft.sourceOpeningTreatment) {
      localErrors.source_opening_treatment = 'Indica cómo resolver la fecha respecto del saldo inicial.'
    }
    if (needsDestinationOpeningChoice.value && !draft.destinationOpeningTreatment) {
      localErrors.destination_opening_treatment = 'Indica cómo resolver la fecha respecto del saldo inicial.'
    }
    if (transferIsCrossCurrency.value && !validEvidence()) {
      localErrors.conversion_evidence = 'Incluye evidencia de la conversión en JSON válido.'
    }
  } else {
    if (!draft.walletId) localErrors.wallet_id = 'Selecciona una billetera.'
    if (!moneyPattern.test(draft.amount) || /^0(?:\.0+)?$/.test(draft.amount)) {
      localErrors.amount = 'Escribe un monto positivo sin separadores.'
    }
    if (needsWalletOpeningChoice.value && !draft.openingTreatment) {
      localErrors.opening_treatment = 'Indica cómo resolver la fecha respecto del saldo inicial.'
    }
    if (draft.kind === 'refund' && draft.sourceExpenseId && !sourceExpenses.value.some((item) => item.id === draft.sourceExpenseId)) {
      localErrors.source_expense_id = 'Carga la página que contiene el gasto de origen.'
    }
    if (confirmedClpIsRelevant.value && draft.confirmedClpAmount &&
      (!moneyPattern.test(draft.confirmedClpAmount) || /^0(?:\.0+)?$/.test(draft.confirmedClpAmount))) {
      localErrors.confirmed_clp_amount = 'Escribe un valor CLP positivo.'
    }
  }

  if (isEditing.value && !draft.reason.trim()) {
    localErrors.reason = 'Explica brevemente el motivo de la corrección.'
  }
  return Object.keys(localErrors).length === 0
}

function validEvidence(): boolean {
  try {
    const parsed: unknown = JSON.parse(draft.conversionEvidence)
    return typeof parsed === 'object' && parsed !== null
  } catch {
    return false
  }
}

function submit(): void {
  if (!validate()) {
    void nextTick(() => {
      document.querySelector<HTMLElement>(
        'input[aria-invalid="true"], select[aria-invalid="true"], textarea[aria-invalid="true"]',
      )?.focus()
    })
    return
  }
  try {
    const request = buildTransactionRequest(draft, props.transaction?.revision)
    if (!needsWalletOpeningChoice.value) delete request.opening_treatment
    if (!needsSourceOpeningChoice.value) delete request.source_opening_treatment
    if (!needsDestinationOpeningChoice.value) delete request.destination_opening_treatment
    if (draft.kind === 'transfer' && !transferIsCrossCurrency.value) {
      delete request.conversion_evidence
    }
    emit('submit', {
      method: props.transaction ? 'PUT' : 'POST',
      route: props.transaction ? `/api/transactions/${props.transaction.id}` : '/api/transactions',
      body: JSON.stringify(request),
    })
  } catch (error) {
    localErrors.conversion_evidence =
      error instanceof Error ? error.message : 'No se pudo preparar la conversión.'
  }
}

function formatSignedEffects(transaction: TransactionView): string {
  return transaction.effects
    .map((effect) => `${effect.amount} ${effect.currency}`)
    .join(' / ')
}

function clearForm(): void {
  emit('cancel')
}
</script>

<template>
  <section class="transaction-editor" :aria-labelledby="isEditing ? 'edit-transaction-title' : 'new-transaction-title'">
    <header class="editor-heading">
      <div>
        <p class="eyebrow">{{ isEditing ? 'Revisión ' + props.transaction?.revision : 'Nuevo movimiento' }}</p>
        <h2 :id="isEditing ? 'edit-transaction-title' : 'new-transaction-title'">
          {{ isEditing ? 'Corregir movimiento' : 'Registrar movimiento' }}
        </h2>
      </div>
    </header>

    <form novalidate @submit.prevent="submit">
      <div class="field-grid">
        <div class="field">
          <label for="transaction-kind">Tipo</label>
          <select id="transaction-kind" v-model="draft.kind">
            <option value="income">Ingreso</option>
            <option value="expense">Gasto</option>
            <option value="refund">Reembolso</option>
            <option value="transfer">Transferencia</option>
          </select>
        </div>
        <div class="field">
          <label for="transaction-date">Fecha real</label>
          <input id="transaction-date" v-model="draft.businessDate" type="date" required :aria-invalid="Boolean(messageFor('business_date'))" :aria-describedby="messageFor('business_date') ? errorId('business_date') : undefined" />
          <p v-if="messageFor('business_date')" :id="errorId('business_date')" class="field-error" role="alert">{{ messageFor('business_date') }}</p>
        </div>
      </div>

      <template v-if="draft.kind === 'transfer'">
        <div class="field-grid">
          <div class="field">
            <label for="source-wallet">Desde</label>
            <select id="source-wallet" v-model="draft.sourceWalletId" required :aria-invalid="Boolean(messageFor('source_wallet_id'))" :aria-describedby="messageFor('source_wallet_id') ? errorId('source_wallet_id') : undefined">
              <option value="">Selecciona origen</option>
              <option v-for="wallet in allWallets" :key="wallet.id" :value="wallet.id">
                {{ wallet.accountName }} · {{ wallet.currency }}
              </option>
            </select>
            <p v-if="messageFor('source_wallet_id')" :id="errorId('source_wallet_id')" class="field-error" role="alert">{{ messageFor('source_wallet_id') }}</p>
          </div>
          <div class="field">
            <label for="source-amount">Monto de origen ({{ selectedSourceWallet?.currency ?? 'moneda' }})</label>
            <input id="source-amount" v-model="draft.sourceAmount" inputmode="decimal" autocomplete="off" required :aria-invalid="Boolean(messageFor('source_amount'))" :aria-describedby="messageFor('source_amount') ? errorId('source_amount') : undefined" />
            <p v-if="messageFor('source_amount')" :id="errorId('source_amount')" class="field-error" role="alert">{{ messageFor('source_amount') }}</p>
          </div>
          <OpeningTreatmentField
            v-if="needsSourceOpeningChoice"
            id="source-opening-treatment"
            v-model="draft.sourceOpeningTreatment"
            :error="messageFor('source_opening_treatment')"
            label="Fecha de origen respecto del saldo inicial"
          />
        </div>

        <div class="field-grid">
          <div class="field">
            <label for="destination-wallet">Hacia</label>
            <select id="destination-wallet" v-model="draft.destinationWalletId" required :aria-invalid="Boolean(messageFor('destination_wallet_id'))" :aria-describedby="messageFor('destination_wallet_id') ? errorId('destination_wallet_id') : undefined">
              <option value="">Selecciona destino</option>
              <option v-for="wallet in allWallets" :key="wallet.id" :value="wallet.id">
                {{ wallet.accountName }} · {{ wallet.currency }}
              </option>
            </select>
            <p v-if="messageFor('destination_wallet_id')" :id="errorId('destination_wallet_id')" class="field-error" role="alert">{{ messageFor('destination_wallet_id') }}</p>
          </div>
          <div class="field">
            <label for="destination-amount">Monto recibido ({{ selectedDestinationWallet?.currency ?? 'moneda' }})</label>
            <input id="destination-amount" v-model="draft.destinationAmount" inputmode="decimal" autocomplete="off" required :aria-invalid="Boolean(messageFor('destination_amount'))" :aria-describedby="messageFor('destination_amount') ? errorId('destination_amount') : undefined" />
            <p v-if="messageFor('destination_amount')" :id="errorId('destination_amount')" class="field-error" role="alert">{{ messageFor('destination_amount') }}</p>
          </div>
          <OpeningTreatmentField
            v-if="needsDestinationOpeningChoice"
            id="destination-opening-treatment"
            v-model="draft.destinationOpeningTreatment"
            :error="messageFor('destination_opening_treatment')"
            label="Fecha de destino respecto del saldo inicial"
          />
        </div>

        <div v-if="transferIsCrossCurrency" class="field">
          <label for="conversion-evidence">Evidencia de conversión (JSON)</label>
          <textarea id="conversion-evidence" v-model="draft.conversionEvidence" rows="3" required :aria-invalid="Boolean(messageFor('conversion_evidence'))" :aria-describedby="messageFor('conversion_evidence') ? `conversion-evidence-hint ${errorId('conversion_evidence')}` : 'conversion-evidence-hint'" />
          <p id="conversion-evidence-hint" class="field-hint">Guarda una referencia legible al comprobante o tipo de cambio usado.</p>
          <p v-if="messageFor('conversion_evidence')" :id="errorId('conversion_evidence')" class="field-error" role="alert">{{ messageFor('conversion_evidence') }}</p>
        </div>
        <p v-else-if="draft.sourceWalletId && draft.destinationWalletId" class="field-hint">
          Las transferencias entre billeteras de la misma moneda deben conservar montos iguales.
        </p>
      </template>

      <template v-else>
        <div class="field-grid">
          <div class="field">
            <label for="transaction-wallet">Billetera</label>
            <select id="transaction-wallet" v-model="draft.walletId" required :aria-invalid="Boolean(messageFor('wallet_id'))" :aria-describedby="messageFor('wallet_id') ? errorId('wallet_id') : undefined">
              <option value="">Selecciona una billetera</option>
              <option v-for="wallet in allWallets" :key="wallet.id" :value="wallet.id">
                {{ wallet.accountName }} · {{ wallet.currency }}
              </option>
            </select>
            <p v-if="messageFor('wallet_id')" :id="errorId('wallet_id')" class="field-error" role="alert">{{ messageFor('wallet_id') }}</p>
          </div>
          <div class="field">
            <label for="transaction-amount">Monto positivo ({{ selectedWallet?.currency ?? 'moneda' }})</label>
            <input id="transaction-amount" v-model="draft.amount" inputmode="decimal" autocomplete="off" required :aria-invalid="Boolean(messageFor('amount'))" :aria-describedby="messageFor('amount') ? errorId('amount') : undefined" />
            <p v-if="messageFor('amount')" :id="errorId('amount')" class="field-error" role="alert">{{ messageFor('amount') }}</p>
          </div>
        </div>
        <OpeningTreatmentField
          v-if="needsWalletOpeningChoice"
          id="wallet-opening-treatment"
          v-model="draft.openingTreatment"
          :error="messageFor('opening_treatment')"
          label="Fecha del movimiento respecto del saldo inicial"
        />

        <div v-if="showCategory" class="field">
          <label for="transaction-category">Categoría (opcional)</label>
          <select id="transaction-category" v-model="draft.categoryId" :aria-invalid="Boolean(messageFor('category_id'))" :aria-describedby="messageFor('category_id') ? errorId('category_id') : undefined">
            <option value="">Sin categoría</option>
            <option v-for="category in categories.filter((item) => !item.archived)" :key="category.id" :value="category.id">
              {{ category.name }}
            </option>
          </select>
          <p v-if="messageFor('category_id')" :id="errorId('category_id')" class="field-error" role="alert">{{ messageFor('category_id') }}</p>
        </div>

        <div v-if="draft.kind === 'refund'" class="field">
          <label for="refund-source">Gasto de origen (opcional)</label>
          <select id="refund-source" v-model="draft.sourceExpenseId" :aria-invalid="Boolean(messageFor('source_expense_id'))" :aria-describedby="messageFor('source_expense_id') ? errorId('source_expense_id') : undefined">
            <option value="">Sin vincular</option>
            <option v-for="expense in sourceExpenses" :key="expense.id" :value="expense.id">
              {{ expense.business_date }} · {{ expense.payee || 'Gasto' }} · {{ formatSignedEffects(expense) }}
            </option>
          </select>
          <p class="field-hint">Si lo vinculas, el reembolso sigue la categoría del gasto original.</p>
          <p v-if="messageFor('source_expense_id')" :id="errorId('source_expense_id')" class="field-error" role="alert">{{ messageFor('source_expense_id') }}</p>
        </div>

        <div v-if="confirmedClpIsRelevant" class="field">
          <label for="confirmed-clp">Valor confirmado en CLP (opcional)</label>
          <input id="confirmed-clp" v-model="draft.confirmedClpAmount" inputmode="decimal" autocomplete="off" :aria-invalid="Boolean(messageFor('confirmed_clp_amount'))" :aria-describedby="messageFor('confirmed_clp_amount') ? `confirmed-clp-hint ${errorId('confirmed_clp_amount')}` : 'confirmed-clp-hint'" />
          <p id="confirmed-clp-hint" class="field-hint">Usa el monto CLP del estado de cuenta. Si lo dejas vacío, el gasto se conserva en su moneda y los valores CLP del presupuesto quedan incompletos.</p>
          <p v-if="messageFor('confirmed_clp_amount')" :id="errorId('confirmed_clp_amount')" class="field-error" role="alert">{{ messageFor('confirmed_clp_amount') }}</p>
        </div>
      </template>

      <div class="field-grid">
        <div class="field">
          <label for="transaction-payee">Comercio o persona (opcional)</label>
          <input id="transaction-payee" v-model="draft.payee" autocomplete="off" />
        </div>
        <div class="field">
          <label for="transaction-note">Nota (opcional)</label>
          <input id="transaction-note" v-model="draft.note" autocomplete="off" />
        </div>
      </div>

      <div v-if="isEditing" class="field">
        <label for="correction-reason">Motivo de la corrección</label>
        <textarea id="correction-reason" v-model="draft.reason" rows="2" required :aria-invalid="Boolean(messageFor('reason'))" :aria-describedby="messageFor('reason') ? errorId('reason') : undefined" />
        <p v-if="messageFor('reason')" :id="errorId('reason')" class="field-error" role="alert">{{ messageFor('reason') }}</p>
      </div>

      <footer class="form-actions">
        <button type="submit" class="primary-button" :disabled="saving || accounts.length === 0">
          {{ saving ? 'Guardando…' : isEditing ? 'Guardar corrección' : 'Registrar movimiento' }}
        </button>
        <button type="button" class="secondary-button" :disabled="saving" @click="clearForm">
          {{ isEditing ? 'Cancelar corrección' : 'Limpiar formulario' }}
        </button>
      </footer>
    </form>
  </section>
</template>

<style scoped>
.transaction-editor { display: grid; gap: 22px; }
.editor-heading h2 { margin: 0; font-size: 24px; }
.field-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.field { display: grid; gap: 7px; margin: 0 0 16px; }
.field label, .opening-choice legend { font-size: 14px; font-weight: 650; }
.field input, .field select, .field textarea { width: 100%; min-height: 42px; border: 1px solid var(--border-strong); border-radius: var(--control-radius); padding: 9px 11px; color: var(--text); background: var(--surface-raised); font: inherit; }
.field textarea { resize: vertical; }
.field-error { margin: 0; color: var(--danger); font-size: 13px; }
.field-hint { margin: 0; color: var(--muted); font-size: 13px; line-height: 1.45; }
.opening-choice { border: 1px solid var(--border-strong); border-radius: var(--control-radius); padding: 12px 14px; }
.opening-choice label { display: flex; align-items: flex-start; gap: 8px; font-weight: 450; line-height: 1.4; }
.opening-choice input { width: auto; min-height: auto; margin-top: 3px; }
.form-actions { display: flex; flex-wrap: wrap; gap: 10px; }
.primary-button, .secondary-button { min-height: 42px; border-radius: var(--control-radius); padding: 9px 14px; font: inherit; font-weight: 650; cursor: pointer; }
.primary-button { color: var(--on-accent); border: 1px solid var(--accent); background: var(--accent); }
.secondary-button { color: var(--text); border: 1px solid var(--border-strong); background: transparent; }
button:disabled { opacity: .6; cursor: not-allowed; }
@media (max-width: 640px) { .field-grid { grid-template-columns: 1fr; gap: 0; } }
input[inputmode="decimal"], input[type="date"] { font-family: var(--font-numeric); }
</style>
