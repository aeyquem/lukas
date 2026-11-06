<script setup lang="ts">
import { computed } from 'vue'
import { canAddCurrencyWallet, walletCountError, type AccountType } from '../account-model'
import { newWallet, isNegativeNonZero, type WalletDraft } from '../account-draft'
import { fieldMessage } from '../composables/use-account-form'
const props = defineProps<{
  accountType: AccountType
  budgetEligible: boolean
  fieldErrors: Record<string, string>
}>()
const wallets = defineModel<WalletDraft[]>({ required: true })
const fieldError = (name: string) => props.fieldErrors[name]
const walletTypeError = computed(() => {
  const code = walletCountError(props.accountType, wallets.value.length)
  return code ? fieldMessage(code) : ''
})
function needsLiabilityBasis(wallet: WalletDraft) {
  return props.budgetEligible && wallet.currency !== 'CLP' && isNegativeNonZero(wallet.opening_amount)
}
function addWallet() {
  if (canAddCurrencyWallet(props.accountType))
    wallets.value = [...wallets.value, newWallet()]
}
function removeWallet(index: number) {
  if (wallets.value.length > 1)
    wallets.value = wallets.value.filter((_, position) => position !== index)
}
</script>
<template>
<fieldset class="wallet-fieldset" :aria-invalid="Boolean(walletTypeError || fieldError('wallets'))" :aria-describedby="walletTypeError || fieldError('wallets') ? 'wallet-count-error' : undefined">
  <legend>Saldos iniciales por moneda</legend>
  <p class="help-copy">{{ canAddCurrencyWallet(accountType) ? 'Puedes registrar saldos en varias monedas. Cada monto conserva su moneda y fecha original.' : 'Esta cuenta usa una moneda. Para otra moneda, crea otra cuenta del mismo tipo.' }}</p>
  <div v-for="(wallet, index) in wallets" :key="index" class="wallet-draft">
    <div class="wallet-draft-title">
      <h3>Moneda {{ index + 1 }}</h3>
      <button v-if="wallets.length > 1" type="button" class="text-button" @click="removeWallet(index)">Quitar</button>
    </div>
    <div class="form-grid wallet-grid">
      <div class="field">
        <label :for="`wallet-currency-${index}`">Moneda</label>
        <input :id="`wallet-currency-${index}`" v-model="wallet.currency" maxlength="3" autocapitalize="characters" spellcheck="false" required :aria-invalid="Boolean(fieldError(`wallets[${index}].currency`))" :aria-describedby="fieldError(`wallets[${index}].currency`) ? `wallet-currency-error-${index}` : undefined" @input="wallet.currency = wallet.currency.toUpperCase()" />
        <p v-if="fieldError(`wallets[${index}].currency`)" :id="`wallet-currency-error-${index}`" class="field-error">{{ fieldError(`wallets[${index}].currency`) }}</p>
      </div>
      <div class="field">
        <label :for="`wallet-amount-${index}`">Saldo inicial</label>
        <input :id="`wallet-amount-${index}`" v-model="wallet.opening_amount" inputmode="decimal" autocomplete="off" required :aria-invalid="Boolean(fieldError(`wallets[${index}].opening_amount`))" :aria-describedby="fieldError(`wallets[${index}].opening_amount`) ? `wallet-amount-error-${index}` : undefined" />
        <p v-if="fieldError(`wallets[${index}].opening_amount`)" :id="`wallet-amount-error-${index}`" class="field-error">{{ fieldError(`wallets[${index}].opening_amount`) }}</p>
      </div>
      <div class="field">
        <label :for="`wallet-date-${index}`">Fecha del saldo</label>
        <input :id="`wallet-date-${index}`" v-model="wallet.opening_date" type="date" required :aria-invalid="Boolean(fieldError(`wallets[${index}].opening_date`))" :aria-describedby="fieldError(`wallets[${index}].opening_date`) ? `wallet-date-error-${index}` : undefined" />
        <p v-if="fieldError(`wallets[${index}].opening_date`)" :id="`wallet-date-error-${index}`" class="field-error">{{ fieldError(`wallets[${index}].opening_date`) }}</p>
      </div>
      <div v-if="needsLiabilityBasis(wallet)" class="field">
        <label :for="`wallet-basis-${index}`">Base de financiamiento en CLP</label>
        <input :id="`wallet-basis-${index}`" v-model="wallet.liability_clp_basis" inputmode="decimal" autocomplete="off" required :aria-invalid="Boolean(fieldError(`wallets[${index}].liability_clp_basis`))" :aria-describedby="fieldError(`wallets[${index}].liability_clp_basis`) ? `wallet-basis-error-${index}` : undefined" />
        <p class="help-copy">Se usa este valor firmado para el presupuesto; no se sustituye por una tasa diaria.</p>
        <p v-if="fieldError(`wallets[${index}].liability_clp_basis`)" :id="`wallet-basis-error-${index}`" class="field-error">{{ fieldError(`wallets[${index}].liability_clp_basis`) }}</p>
      </div>
    </div>
  </div>
  <p v-if="walletTypeError" id="wallet-count-error" class="field-error" role="status">{{ walletTypeError }}</p>
  <p v-else-if="fieldError('wallets')" id="wallet-count-error" class="field-error" role="status">{{ fieldError('wallets') }}</p>
  <button v-if="canAddCurrencyWallet(accountType)" type="button" class="secondary-button" @click="addWallet">Añadir otra moneda</button>
</fieldset>
</template>
<style scoped>
.wallet-draft-title {
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



.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.field {
  display: grid;
  align-content: start;
  gap: 7px;
  margin-bottom: 16px;
}

.field label,
.wallet-fieldset legend {
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

.wallet-fieldset {
  min-width: 0;
  margin: 22px 0 0;
  padding: 18px;
  border: 1px solid var(--border);
  border-radius: var(--control-radius);
}

.wallet-fieldset legend {
  padding-inline: 8px;
}

.wallet-fieldset > .help-copy {
  margin-bottom: 16px;
  font-size: 13px;
}

.wallet-draft {
  margin-bottom: 14px;
  padding: 16px;
  border-radius: var(--control-radius);
  background: var(--surface-soft);
}

.wallet-draft-title {
  margin-bottom: 14px;
}

.wallet-draft-title h3 {
  margin: 0;
}

.wallet-grid .field {
  margin-bottom: 8px;
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

.secondary-button {
  border-color: var(--border-strong);
  color: var(--accent);
  background: var(--surface-soft);
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

@media (max-width: 760px) {
.form-grid {
    grid-template-columns: 1fr;
  }
}

.field input[inputmode="decimal"],
.field input[type="date"] {
  font-family: var(--font-numeric);
}
</style>
