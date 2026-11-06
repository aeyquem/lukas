<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import AccountCard from '../components/accounts/AccountCard.vue'
import {
  canAddCurrencyWallet,
  setDraftAccountType,
  walletCountError,
  type AccountType,
} from '../components/accounts/account-model'
import { listPendingAccountWrites, writeThenRead } from '../components/accounts/account-write-flow'
import { ApiClientError, apiRequest } from '../shared/api/client'
import {
  dismissPendingWrite,
  PendingWriteStore,
  type PendingFinancialWrite,
  retryPendingWrite,
  sendFinancialWrite,
} from '../shared/api/pending-commands'
import { formatMoney } from '../shared/formatting/money'
import { formatBusinessDate } from '../shared/formatting/business-date'

interface OpeningView {
  id: string
  amount: string
  business_date: string
  revision: number
  liability_clp_basis: string | null
  liability_basis_provenance: string | null
  liability_basis_revision: number | null
}

interface WalletView {
  id: string
  currency: string
  balance: string
  openings: OpeningView[]
}

interface AccountView {
  id: string
  name: string
  account_type: AccountType
  base_currency: string
  budget_eligible: boolean
  revision: number
  wallets: WalletView[]
}

interface WalletDraft {
  currency: string
  opening_amount: string
  opening_date: string
  liability_clp_basis: string
}

interface AccountDraft {
  name: string
  account_type: AccountType
  base_currency: string
  budget_eligible: boolean
  wallets: WalletDraft[]
}

interface MutationReceipt {
  resource_id: string
  revision: number
}

const accountTypes: { value: AccountType; label: string }[] = [
  { value: 'bank', label: 'Banco' },
  { value: 'cash', label: 'Efectivo' },
  { value: 'savings', label: 'Ahorro' },
  { value: 'card', label: 'Tarjeta' },
  { value: 'investment', label: 'Inversión' },
]

const isoCurrency = /^[A-Z]{3}$/
const plainMoney = /^-?(0|[1-9][0-9]*)(?:\.([0-9]+))?$/
const maxMoneyMantissa = 79228162514264337593543950335n

function isValidMoney(value: string): boolean {
  const match = plainMoney.exec(value)
  if (!match || (match[2]?.length ?? 0) > 28) return false
  const digits = value.replace(/^-/, '').replace('.', '')
  return BigInt(digits) <= maxMoneyMantissa
}

function todayInSantiago(): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: 'America/Santiago',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).formatToParts(new Date())
  const part = (type: string) => parts.find((item) => item.type === type)?.value ?? ''
  return `${part('year')}-${part('month')}-${part('day')}`
}

function newWallet(): WalletDraft {
  return {
    currency: 'CLP',
    opening_amount: '0',
    opening_date: todayInSantiago(),
    liability_clp_basis: '',
  }
}

function newDraft(): AccountDraft {
  return {
    name: '',
    account_type: 'bank',
    base_currency: 'CLP',
    budget_eligible: true,
    wallets: [newWallet()],
  }
}

function isNegativeNonZero(value: string): boolean {
  return value.startsWith('-') && !/^-(?:0+)(?:\.0+)?$/.test(value)
}

function signOf(value: string): -1 | 0 | 1 {
  if (/^-?(?:0+)(?:\.0+)?$/.test(value)) return 0
  return value.startsWith('-') ? -1 : 1
}

const accounts = ref<AccountView[]>([])
const draft = reactive<AccountDraft>(newDraft())
const selectedAccount = ref<AccountView | null>(null)
const editName = ref('')
const openingCorrection = ref<{ walletId: string; opening: OpeningView } | null>(null)
const correctionAmount = ref('')
const correctionBasis = ref('')
const correctionReason = ref('')
const fieldErrors = reactive<Record<string, string>>({})
const pageError = ref('')
const pendingError = ref('')
const successMessage = ref('')
const isLoading = ref(false)
const isSaving = ref(false)
const pendingAccountWrites = ref<PendingFinancialWrite[]>([])
const dismissedPendingKeys = ref(new Set<string>())
const isEditing = computed(() => selectedAccount.value !== null)
const walletTypeError = computed(() => {
  const code = walletCountError(draft.account_type, draft.wallets.length)
  return code ? fieldMessage(code) : ''
})
const needsLiabilityBasis = (wallet: WalletDraft) =>
  draft.budget_eligible && wallet.currency !== 'CLP' && isNegativeNonZero(wallet.opening_amount)
const needsCorrectionBasis = computed(() => {
  const account = selectedAccount.value
  const correction = openingCorrection.value
  if (!account || !correction) return false
  const wallet = account.wallets.find((item) => item.id === correction.walletId)
  return Boolean(
    wallet &&
      account.budget_eligible &&
      wallet.currency !== 'CLP' &&
      (correction.opening.liability_clp_basis !== null || isNegativeNonZero(correctionAmount.value)),
  )
})

watch(
  () => draft.account_type,
  (type, previous) => {
    if (type === 'investment' && previous !== 'investment') draft.budget_eligible = false
  },
)

onMounted(() => {
  void loadAccounts()
  void loadPendingAccountWrites()
})

function loadPendingAccountWrites(): void {
  try {
    pendingAccountWrites.value = listPendingAccountWrites(new PendingWriteStore())
    pendingError.value = ''
  } catch (error) {
    pendingError.value = error instanceof Error
      ? error.message
      : 'No se pudieron recuperar los comandos pendientes.'
  }
}

async function loadAccounts(): Promise<void> {
  isLoading.value = true
  pageError.value = ''
  try {
    accounts.value = await apiRequest<AccountView[]>('accounts')
  } catch (error) {
    pageError.value = errorMessage(error)
  } finally {
    isLoading.value = false
  }
}

async function refreshAccounts(): Promise<void> {
  await loadAccounts()
  const selectedId = selectedAccount.value?.id
  if (!selectedId || pageError.value) return
  try {
    await refreshAccount(selectedId)
  } catch (error) {
    pageError.value = errorMessage(error)
  }
}

function clearErrors(): void {
  for (const key of Object.keys(fieldErrors)) delete fieldErrors[key]
  pageError.value = ''
  successMessage.value = ''
}

function fieldError(name: string): string | undefined {
  return fieldErrors[name]
}

function receiveError(error: unknown): void {
  const pendingCommandKey =
    typeof error === 'object' && error !== null && 'pendingCommandKey' in error
      ? (error as { pendingCommandKey?: unknown }).pendingCommandKey
      : undefined
  if (typeof pendingCommandKey === 'string') {
    loadPendingAccountWrites()
    dismissedPendingKeys.value.delete(pendingCommandKey)
  }
  if (error instanceof ApiClientError) {
    for (const item of error.fields) fieldErrors[item.field] = fieldMessage(item.code)
    pageError.value = error.outcomeUnknown
      ? 'No se pudo confirmar el resultado. Actualiza la lista para comprobar si la cuenta se guardó.'
      : error.fields.length > 0
        ? ''
        : error.message
    return
  }
  pageError.value = error instanceof Error
    ? error.message
    : 'No fue posible guardar los cambios. Inténtalo de nuevo.'
}

async function sendAccountWrite<T>(
  method: 'POST' | 'PUT',
  route: string,
  payload: unknown,
): Promise<T> {
  return sendFinancialWrite<T>({
    method,
    route,
    body: JSON.stringify(payload),
  })
}

async function retryAccountWrite(command: PendingFinancialWrite): Promise<void> {
  clearErrors()
  isSaving.value = true
  let outcome: Awaited<ReturnType<typeof writeThenRead<MutationReceipt, AccountView>>>
  try {
    outcome = await writeThenRead(
      () => retryPendingWrite<MutationReceipt>(command.idempotencyKey),
      (receipt) => command.route === '/api/accounts'
        ? apiRequest<AccountView>(`accounts/${receipt.resource_id}`)
        : apiRequest<AccountView>(`accounts/${command.route.split('/')[3] || receipt.resource_id}`),
    )
    loadPendingAccountWrites()
  } catch (error) {
    loadPendingAccountWrites()
    receiveError(error)
    isSaving.value = false
    return
  }

  const { receipt, value: authoritative } = outcome
  // The receipt is final. A failed read is reported separately and never
  // makes the command retryable again.
  if (command.route === '/api/accounts') {
    Object.assign(draft, newDraft())
    successMessage.value = `La cuenta quedó creada (ID ${receipt.resource_id}).`
    if (authoritative) {
      accounts.value = [authoritative, ...accounts.value.filter((item) => item.id !== authoritative.id)]
    }
  } else {
    successMessage.value = `La operación quedó confirmada (revisión ${receipt.revision}).`
    if (authoritative) {
      accounts.value = accounts.value.map((item) => item.id === authoritative.id ? authoritative : item)
      if (selectedAccount.value?.id === authoritative.id) selectedAccount.value = authoritative
    }
  }
  if (outcome.readError !== undefined) {
    pageError.value = 'La operación quedó confirmada, pero no se pudo actualizar la lectura. Usa “Actualizar cuentas”; no vuelvas a enviar la operación.'
  }
  isSaving.value = false
}

function dismissAccountWrite(command: PendingFinancialWrite): void {
  try {
    dismissPendingWrite(command.idempotencyKey)
    dismissedPendingKeys.value.add(command.idempotencyKey)
  } catch (error) {
    pageError.value = error instanceof Error ? error.message : 'No se pudo ocultar el aviso.'
  }
}

function pendingWriteLabel(command: PendingFinancialWrite): string {
  if (command.route === '/api/accounts') return 'Crear cuenta'
  if (command.route.includes('/openings/')) return 'Corregir saldo inicial'
  return 'Editar nombre de cuenta'
}

function errorMessage(error: unknown): string {
  return error instanceof ApiClientError
    ? error.message
    : 'No fue posible cargar las cuentas. Comprueba la conexión e inténtalo de nuevo.'
}

function fieldMessage(code: string): string {
  const messages: Record<string, string> = {
    required: 'Este campo es obligatorio.',
    unsupported_value: 'Selecciona un tipo de cuenta válido.',
    must_be_iso_style_code: 'Usa un código de moneda de tres letras, por ejemplo CLP.',
    at_least_one_required: 'Añade al menos una moneda.',
    duplicate_currency: 'Esta cuenta ya tiene una billetera en esa moneda.',
    multiple_currencies_unsupported_for_account_type: 'Este tipo de cuenta admite una sola moneda. Quita las demás para guardar.',
    required_for_foreign_liability: 'Indica la base de financiamiento CLP para esta deuda.',
    must_match_liability_sign: 'La base CLP debe tener el mismo signo que la deuda.',
    only_valid_for_eligible_foreign_liability: 'Este valor solo se usa para una deuda elegible en moneda extranjera.',
    outside_decimal_envelope: 'El monto excede el rango decimal admitido.',
    stale_revision: 'La cuenta cambió. Vuelve a cargarla antes de guardar.',
  }
  return messages[code] ?? 'Revisa este valor.'
}

function validateDraft(): boolean {
  clearErrors()
  if (!draft.name.trim()) fieldErrors.name = 'Escribe un nombre para la cuenta.'
  if (!isoCurrency.test(draft.base_currency)) {
    fieldErrors.base_currency = 'Usa un código de moneda de tres letras, por ejemplo CLP.'
  }
  const walletError = walletCountError(draft.account_type, draft.wallets.length)
  if (walletError) fieldErrors.wallets = fieldMessage(walletError)

  const usedCurrencies = new Set<string>()
  draft.wallets.forEach((wallet, index) => {
    if (!isoCurrency.test(wallet.currency)) {
      fieldErrors[`wallets[${index}].currency`] = 'Usa un código de moneda de tres letras.'
    } else if (usedCurrencies.has(wallet.currency)) {
      fieldErrors[`wallets[${index}].currency`] = 'Esta cuenta ya tiene una billetera en esa moneda.'
    }
    usedCurrencies.add(wallet.currency)

    if (!isValidMoney(wallet.opening_amount)) {
      fieldErrors[`wallets[${index}].opening_amount`] = 'Escribe un decimal válido con hasta 28 decimales y dentro del rango permitido.'
    }
    if (!wallet.opening_date) {
      fieldErrors[`wallets[${index}].opening_date`] = 'Selecciona la fecha del saldo inicial.'
    }
    if (needsLiabilityBasis(wallet)) {
      if (!isValidMoney(wallet.liability_clp_basis)) {
        fieldErrors[`wallets[${index}].liability_clp_basis`] = 'Indica la base CLP firmada.'
      } else if (!isNegativeNonZero(wallet.liability_clp_basis)) {
        fieldErrors[`wallets[${index}].liability_clp_basis`] = 'La base CLP debe ser negativa.'
      }
    }
  })
  return Object.keys(fieldErrors).length === 0
}

async function submitCreate(): Promise<void> {
  if (!validateDraft()) return
  isSaving.value = true
  let outcome: Awaited<ReturnType<typeof writeThenRead<MutationReceipt, AccountView>>>
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
    outcome = await writeThenRead(
      () => sendAccountWrite<MutationReceipt>('POST', '/api/accounts', payload),
      (receipt) => apiRequest<AccountView>(`accounts/${receipt.resource_id}`),
    )
  } catch (error) {
    await loadAccounts()
    receiveError(error)
    isSaving.value = false
    return
  }

  const { receipt, value: account } = outcome
  Object.assign(draft, newDraft())
  successMessage.value = `La cuenta quedó creada (ID ${receipt.resource_id}).`
  if (account) {
    accounts.value = [account, ...accounts.value.filter((item) => item.id !== account.id)]
    successMessage.value = 'La cuenta se creó y sus saldos quedaron registrados.'
  } else {
    pageError.value = 'La cuenta quedó creada, pero no se pudo actualizar su lectura. Usa “Actualizar cuentas”; no vuelvas a enviar la creación.'
  }
  isSaving.value = false
}

function startCreate(): void {
  selectedAccount.value = null
  openingCorrection.value = null
  Object.assign(draft, newDraft())
  clearErrors()
}

async function startEdit(account: AccountView): Promise<void> {
  selectedAccount.value = account
  editName.value = account.name
  openingCorrection.value = null
  clearErrors()
  isLoading.value = true
  try {
    const authoritative = await apiRequest<AccountView>(`accounts/${account.id}`)
    accounts.value = accounts.value.map((item) => (item.id === account.id ? authoritative : item))
    selectedAccount.value = authoritative
    editName.value = authoritative.name
  } catch (error) {
    pageError.value = errorMessage(error)
  } finally {
    isLoading.value = false
  }
}

function addWallet(): void {
  if (canAddCurrencyWallet(draft.account_type)) draft.wallets.push(newWallet())
}

function updateAccountType(event: Event): void {
  const accountType = (event.target as HTMLSelectElement).value as AccountType
  setDraftAccountType(draft, accountType)
}

function removeWallet(index: number): void {
  if (draft.wallets.length > 1) draft.wallets.splice(index, 1)
}

async function submitNameEdit(): Promise<void> {
  const account = selectedAccount.value
  clearErrors()
  if (!account) return
  if (!editName.value.trim()) {
    fieldErrors.name = 'Escribe un nombre para la cuenta.'
    return
  }
  isSaving.value = true
  let outcome: Awaited<ReturnType<typeof writeThenRead<MutationReceipt, AccountView>>>
  try {
    outcome = await writeThenRead(
      () => sendAccountWrite<MutationReceipt>('PUT', `/api/accounts/${account.id}`, {
        name: editName.value.trim(),
        expected_revision: account.revision,
      }),
      () => apiRequest<AccountView>(`accounts/${account.id}`),
    )
  } catch (error) {
    await loadAccounts()
    receiveError(error)
    isSaving.value = false
    return
  }

  const { receipt, value: authoritative } = outcome
  const renamedAccount = authoritative ?? { ...account, name: editName.value.trim(), revision: receipt.revision }
  selectedAccount.value = renamedAccount
  accounts.value = accounts.value.map((item) => (item.id === account.id ? renamedAccount : item))
  successMessage.value = `Nombre guardado (revisión ${receipt.revision}).`
  if (outcome.readError !== undefined) {
    pageError.value = 'El nombre quedó guardado, pero no se pudo actualizar la lectura. Usa “Actualizar cuentas”; no vuelvas a enviar el cambio.'
  }
  isSaving.value = false
}

function startOpeningCorrection(walletId: string, opening: OpeningView): void {
  openingCorrection.value = { walletId, opening }
  correctionAmount.value = opening.amount
  correctionBasis.value = opening.liability_clp_basis ?? ''
  correctionReason.value = ''
  clearErrors()
}

async function submitOpeningCorrection(): Promise<void> {
  const account = selectedAccount.value
  const correction = openingCorrection.value
  clearErrors()
  if (!account || !correction) return
  if (!isValidMoney(correctionAmount.value)) {
    fieldErrors.amount = 'Escribe un decimal válido con hasta 28 decimales y dentro del rango permitido.'
  }
  if (!correctionReason.value.trim()) fieldErrors.reason = 'Explica brevemente el motivo del cambio.'
  const wallet = account.wallets.find((item) => item.id === correction.walletId)
  const basisRequired = Boolean(
    wallet &&
      account.budget_eligible &&
      wallet.currency !== 'CLP' &&
      (correction.opening.liability_clp_basis !== null || isNegativeNonZero(correctionAmount.value)),
  )
  if (basisRequired && !isValidMoney(correctionBasis.value)) {
    fieldErrors.liability_clp_basis = 'Indica la base CLP firmada para esta deuda.'
  }
  if (basisRequired && isValidMoney(correctionBasis.value) && signOf(correctionBasis.value) !== signOf(correctionAmount.value)) {
    fieldErrors.liability_clp_basis = 'La base CLP debe tener el mismo signo que el saldo corregido.'
  }
  if (Object.keys(fieldErrors).length > 0) return

  isSaving.value = true
  let outcome: Awaited<ReturnType<typeof writeThenRead<MutationReceipt, AccountView>>>
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
    outcome = await writeThenRead(
      () => sendAccountWrite<MutationReceipt>(
        'PUT',
        `/api/accounts/${account.id}/openings/${correction.opening.id}`,
        body,
      ),
      () => apiRequest<AccountView>(`accounts/${account.id}`),
    )
  } catch (error) {
    await loadAccounts()
    receiveError(error)
    isSaving.value = false
    return
  }

  const { receipt, readError } = outcome
  openingCorrection.value = null
  successMessage.value = `Corrección guardada (revisión ${receipt.revision}).`
  if (outcome.value) {
    accounts.value = accounts.value.map((item) => item.id === account.id ? outcome.value! : item)
    if (selectedAccount.value?.id === account.id) selectedAccount.value = outcome.value
  }
  if (readError !== undefined) {
    pageError.value = 'La corrección quedó guardada, pero no se pudo actualizar la lectura. Usa “Actualizar cuentas”; no vuelvas a enviar la corrección.'
  }
  isSaving.value = false
}

async function refreshAccount(accountId: string): Promise<void> {
  const authoritative = await apiRequest<AccountView>(`accounts/${accountId}`)
  accounts.value = accounts.value.map((item) => (item.id === accountId ? authoritative : item))
  if (selectedAccount.value?.id === accountId) selectedAccount.value = authoritative
}

async function refreshSelectedAccount(): Promise<void> {
  const account = selectedAccount.value
  if (!account) return
  pageError.value = ''
  try {
    await refreshAccount(account.id)
  } catch (error) {
    pageError.value = errorMessage(error)
  }
}

</script>

<template>
  <main class="page-shell accounts-page" aria-labelledby="accounts-title">
    <header class="page-header">
      <div>
        <p class="eyebrow">Finanzas personales y del hogar</p>
        <h1 id="accounts-title">Cuentas</h1>
        <p class="intro">Conserva cada saldo en su moneda y registra cuándo comenzó a formar parte de tus cuentas.</p>
      </div>
      <button type="button" class="primary-button" @click="startCreate">Nueva cuenta</button>
    </header>

    <p v-if="pageError" class="notice error-notice" role="alert">{{ pageError }}</p>
    <p v-if="successMessage" class="notice success-notice" role="status">{{ successMessage }}</p>
    <p v-if="pendingError" class="notice error-notice" role="alert">No se pudieron revisar las operaciones guardadas: {{ pendingError }}</p>
    <section v-if="pendingAccountWrites.length > 0" class="pending-commands" aria-labelledby="pending-commands-title">
      <h2 id="pending-commands-title">Operaciones por confirmar</h2>
      <p class="help-copy">Algunas operaciones todavía no tienen un resultado confirmado. Puedes reintentarlas de forma segura o cerrar el aviso; seguirán disponibles hasta que se confirme el resultado.</p>
      <ul>
        <li v-for="command in pendingAccountWrites" :key="command.idempotencyKey" class="pending-command-row">
          <div>
            <strong>{{ pendingWriteLabel(command) }}</strong>
            <p v-if="dismissedPendingKeys.has(command.idempotencyKey)" class="muted">Aviso oculto; el comando sigue guardado y disponible aquí.</p>
            <p v-else class="muted">El resultado todavía no se confirmó.</p>
          </div>
          <div class="pending-actions">
            <button type="button" class="secondary-button" :disabled="isSaving" @click="retryAccountWrite(command)">Reintentar</button>
            <button v-if="!dismissedPendingKeys.has(command.idempotencyKey)" type="button" class="text-button" @click="dismissAccountWrite(command)">Ocultar aviso</button>
          </div>
        </li>
      </ul>
    </section>

    <div class="accounts-layout">
      <section class="account-list" aria-labelledby="account-list-title">
        <div class="section-heading">
          <h2 id="account-list-title">Tus cuentas</h2>
          <div class="list-actions">
            <span v-if="!isLoading" class="count">{{ accounts.length }}</span>
            <button type="button" class="text-button" :disabled="isLoading" @click="refreshAccounts">Actualizar cuentas</button>
          </div>
        </div>
        <p v-if="isLoading" role="status" class="muted">Cargando cuentas…</p>
        <p v-else-if="accounts.length === 0" class="empty-state">Todavía no tienes cuentas. Registra una para ver sus saldos aquí.</p>
        <ul v-else class="account-items">
          <li v-for="account in accounts" :key="account.id">
            <AccountCard :account="account" :selected="selectedAccount?.id === account.id" @select="startEdit(account)" />
          </li>
        </ul>
      </section>

      <section v-if="!isEditing" class="editor-panel" aria-labelledby="create-title">
        <div class="section-heading">
          <div>
            <p class="eyebrow">Nueva cuenta</p>
            <h2 id="create-title">Registra una cuenta</h2>
          </div>
        </div>
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

          <fieldset class="wallet-fieldset" :aria-invalid="Boolean(walletTypeError || fieldError('wallets'))" :aria-describedby="walletTypeError || fieldError('wallets') ? 'wallet-count-error' : undefined">
            <legend>Saldos iniciales por moneda</legend>
            <p class="help-copy">{{ canAddCurrencyWallet(draft.account_type) ? 'Puedes registrar saldos en varias monedas. Cada monto conserva su moneda y fecha original.' : 'Esta cuenta usa una moneda. Para otra moneda, crea otra cuenta del mismo tipo.' }}</p>
            <div v-for="(wallet, index) in draft.wallets" :key="index" class="wallet-draft">
              <div class="wallet-draft-title">
                <h3>Moneda {{ index + 1 }}</h3>
                <button v-if="draft.wallets.length > 1" type="button" class="text-button" @click="removeWallet(index)">Quitar</button>
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
            <button v-if="canAddCurrencyWallet(draft.account_type)" type="button" class="secondary-button" @click="addWallet">Añadir otra moneda</button>
          </fieldset>

          <div class="form-actions">
            <button type="submit" class="primary-button" :disabled="isSaving">{{ isSaving ? 'Guardando…' : 'Guardar cuenta' }}</button>
          </div>
        </form>
      </section>

      <section v-else class="editor-panel" aria-labelledby="edit-title">
        <div class="section-heading">
          <div>
            <p class="eyebrow">Cuenta · revisión {{ selectedAccount?.revision }}</p>
            <h2 id="edit-title">{{ selectedAccount?.name }}</h2>
          </div>
          <button type="button" class="text-button" @click="startCreate">Cerrar</button>
        </div>

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

        <section class="wallet-readbacks" aria-labelledby="wallets-title">
          <div class="section-heading">
            <h3 id="wallets-title">Saldos autoritativos</h3>
            <button type="button" class="text-button" :disabled="isLoading" @click="refreshSelectedAccount">Actualizar</button>
          </div>
          <article v-for="wallet in selectedAccount?.wallets ?? []" :key="wallet.id" class="wallet-readback">
            <div class="readback-heading">
              <h4>{{ wallet.currency }} · {{ formatMoney(wallet.balance, wallet.currency) }}</h4>
            </div>
            <p v-if="wallet.openings.length === 0" class="muted">No hay saldos iniciales registrados.</p>
            <ul v-else class="opening-list">
              <li v-for="opening in wallet.openings" :key="opening.id" class="opening-row">
                <div>
                  <strong>{{ formatMoney(opening.amount, wallet.currency) }}</strong>
                  <span class="opening-date">{{ formatBusinessDate(opening.business_date) }}</span>
                  <small>Revisión {{ opening.revision }}<template v-if="opening.liability_basis_provenance"> · Base {{ opening.liability_basis_provenance }}</template></small>
                  <small v-if="opening.liability_clp_basis">Base CLP: {{ formatMoney(opening.liability_clp_basis, 'CLP') }}</small>
                </div>
                <button type="button" class="text-button" @click="startOpeningCorrection(wallet.id, opening)">Corregir saldo</button>
              </li>
            </ul>
          </article>
        </section>

        <form v-if="openingCorrection" class="correction-panel" @submit.prevent="submitOpeningCorrection" novalidate>
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
            <button type="button" class="secondary-button" @click="openingCorrection = null">Cancelar</button>
            <button type="submit" class="primary-button" :disabled="isSaving">Guardar corrección</button>
          </div>
        </form>
      </section>
    </div>
  </main>
</template>

<style scoped>
.accounts-page {
  padding-block: 42px 80px;
}

.page-header,
.section-heading,
.account-card-heading,
.wallet-draft-title,
.readback-heading,
.form-actions,
.opening-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.page-header {
  align-items: end;
  margin-bottom: 32px;
}

h1,
h2,
h3,
h4,
p {
  margin-top: 0;
}

h1 {
  margin-bottom: 12px;
  font-size: clamp(36px, 5vw, 52px);
  letter-spacing: -0.055em;
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

.intro,
.help-copy,
.muted,
.empty-state,
.account-meta,
.opening-row small {
  color: #616760;
  line-height: 1.5;
}

.intro {
  max-width: 640px;
  margin-bottom: 0;
  font-size: 16px;
}

.eyebrow {
  margin-bottom: 8px;
}

.accounts-layout {
  display: grid;
  grid-template-columns: minmax(250px, 0.78fr) minmax(0, 1.5fr);
  align-items: start;
  gap: 24px;
}

.account-list,
.editor-panel {
  border: 1px solid #e1e2dc;
  border-radius: 18px;
  background: #fff;
  box-shadow: 0 12px 40px rgb(35 46 39 / 4%);
}

.account-list {
  padding: 20px;
}

.editor-panel {
  padding: 24px;
}

.section-heading {
  margin-bottom: 18px;
}

.list-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.count {
  display: grid;
  min-width: 28px;
  height: 28px;
  place-items: center;
  border-radius: 999px;
  color: #52675b;
  background: #edf3ee;
  font-size: 13px;
  font-weight: 700;
}

.account-items,
.opening-list {
  display: grid;
  margin: 0;
  padding: 0;
  list-style: none;
}

.account-items {
  gap: 10px;
}

.account-meta {
  font-size: 12px;
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

.field label,
.wallet-fieldset legend {
  font-size: 13px;
  font-weight: 700;
}

input,
select,
textarea {
  width: 100%;
  min-height: 44px;
  padding: 10px 12px;
  border: 1px solid #cfd5cf;
  border-radius: 10px;
  color: #272b27;
  background: #fff;
  font: inherit;
}

input:focus,
select:focus,
textarea:focus {
  border-color: #34765a;
}

input[aria-invalid='true'],
textarea[aria-invalid='true'] {
  border-color: #b4473f;
}

.eligibility-field {
  display: flex;
  grid-column: 1 / -1;
  align-items: start;
  gap: 12px;
  padding: 14px;
  border: 1px solid #e4e6e0;
  border-radius: 12px;
  cursor: pointer;
}

.eligibility-field input {
  width: 18px;
  min-height: 18px;
  margin: 2px 0 0;
  accent-color: #34765a;
}

.eligibility-field span {
  display: grid;
  gap: 3px;
}

.eligibility-field strong {
  font-size: 13px;
}

.eligibility-field small {
  color: #616760;
  line-height: 1.4;
}

.wallet-fieldset {
  min-width: 0;
  margin: 22px 0 0;
  padding: 18px;
  border: 1px solid #e2e5df;
  border-radius: 14px;
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
  border-radius: 12px;
  background: #f7f8f5;
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
  color: #a3332b;
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
  border-radius: 10px;
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.primary-button {
  color: #fff;
  background: #34765a;
}

.primary-button:hover:not(:disabled) {
  background: #285c45;
}

.secondary-button {
  border-color: #cfd8d0;
  color: #315e46;
  background: #f7faf7;
}

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.text-button {
  border: 0;
  padding: 7px 4px;
  color: #315e46;
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
  border-radius: 12px;
  background: #f7f8f5;
}

.wallet-readbacks {
  margin-top: 24px;
}

.wallet-readback {
  margin-top: 12px;
  padding: 16px;
  border: 1px solid #e4e6e0;
  border-radius: 12px;
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
  border-top: 1px solid #edf0ea;
}

.opening-row > div {
  display: grid;
  gap: 4px;
}

.opening-date {
  color: #616760;
  font-size: 12px;
}

.opening-row small {
  font-size: 11px;
}

.correction-panel {
  margin-top: 22px;
  padding: 20px;
  border: 1px solid #dce7dd;
  border-radius: 14px;
  background: #f7faf7;
}

.notice {
  margin: 0 0 20px;
  padding: 12px 14px;
  border-radius: 10px;
  font-size: 14px;
}

.error-notice {
  color: #8e2d27;
  background: #fff0ee;
}

.success-notice {
  color: #285c45;
  background: #eaf4ec;
}

.empty-state {
  margin-bottom: 0;
  font-size: 13px;
}

@media (max-width: 760px) {
  .accounts-page {
    padding-top: 28px;
  }

  .page-header {
    align-items: start;
    flex-direction: column;
  }

  .accounts-layout {
    grid-template-columns: 1fr;
  }

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
</style>
