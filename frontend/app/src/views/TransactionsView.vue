<script setup lang="ts">
import { onMounted, ref } from 'vue'
import TransactionForm from '../components/activity/TransactionForm.vue'
import TransactionList from '../components/activity/TransactionList.vue'
import { ApiClientError, apiRequest } from '../shared/api/client'
import {
  PendingWriteStore,
  dismissPendingWrite,
  retryPendingWrite,
  sendFinancialWrite,
  type PendingFinancialWrite,
} from '../shared/api/pending-commands'
import type {
  AccountView,
  TransactionCategory,
  TransactionView,
} from '../components/activity/transaction-model'

interface TransactionCursor {
  before_date: string
  before_id: string
}

interface TransactionPage {
  items: TransactionView[]
  next: TransactionCursor | null
}

interface BootstrapResponse {
  categories: TransactionCategory[]
}

interface MutationReceipt {
  resource_id: string
  revision: number
}

const accounts = ref<AccountView[]>([])
const categories = ref<TransactionCategory[]>([])
const transactions = ref<TransactionView[]>([])
const nextCursor = ref<TransactionCursor | null>(null)
const editingTransaction = ref<TransactionView | null>(null)
const pendingCommands = ref<PendingFinancialWrite[]>([])
const dismissedPendingKeys = ref(new Set<string>())
const serverErrors = ref<Record<string, string>>({})
const pageError = ref('')
const successMessage = ref('')
const isLoading = ref(false)
const isLoadingMore = ref(false)
const isSaving = ref(false)
const initialized = ref(false)

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

const today = todayInSantiago()
let pendingStore: PendingWriteStore | undefined

function getPendingStore(): PendingWriteStore {
  pendingStore ??= new PendingWriteStore()
  return pendingStore
}

onMounted(() => {
  restorePendingCommands()
  void loadInitialData()
})

async function loadInitialData(): Promise<void> {
  isLoading.value = true
  pageError.value = ''
  try {
    const [accountRows, bootstrap] = await Promise.all([
      apiRequest<AccountView[]>('accounts'),
      apiRequest<BootstrapResponse>('bootstrap'),
    ])
    accounts.value = accountRows
    categories.value = bootstrap.categories
    await loadFirstPage()
    initialized.value = true
  } catch (error) {
    pageError.value = messageFor(error, 'No fue posible cargar los movimientos.')
  } finally {
    isLoading.value = false
  }
}

async function loadFirstPage(): Promise<void> {
  const page = await fetchPage(null)
  transactions.value = page.items
  nextCursor.value = page.next
}

async function loadMore(): Promise<void> {
  const cursor = nextCursor.value
  if (!cursor || isLoadingMore.value) return
  isLoadingMore.value = true
  pageError.value = ''
  try {
    const page = await fetchPage(cursor)
    transactions.value = [...transactions.value, ...page.items]
    nextCursor.value = page.next
  } catch (error) {
    pageError.value = messageFor(error, 'No fue posible cargar la página siguiente.')
  } finally {
    isLoadingMore.value = false
  }
}

async function fetchPage(cursor: TransactionCursor | null): Promise<TransactionPage> {
  const params = new URLSearchParams({ page_size: '50' })
  if (cursor) {
    params.set('before_date', cursor.before_date)
    params.set('before_id', cursor.before_id)
  }
  return apiRequest<TransactionPage>(`transactions?${params.toString()}`)
}

async function submitWrite(command: { method: 'POST' | 'PUT'; route: string; body: string }): Promise<void> {
  pageError.value = ''
  successMessage.value = ''
  serverErrors.value = {}
  isSaving.value = true
  try {
    const receipt = await sendFinancialWrite<MutationReceipt>(command)
    editingTransaction.value = null
    successMessage.value = `Movimiento guardado (revisión ${receipt.revision}).`
    restorePendingCommands()
    try {
      await loadFirstPage()
    } catch {
      pageError.value = 'El movimiento se guardó, pero no se pudo actualizar la lista. Intenta recargarla.'
    }
  } catch (error) {
    if (error instanceof ApiClientError && error.fields.length > 0) {
      serverErrors.value = Object.fromEntries(error.fields.map((field) => [field.field, fieldMessage(field.code)]))
    }
    if (hasPendingCommandKey(error)) restorePendingCommands()
    pageError.value = messageFor(error, 'No fue posible guardar el movimiento.')
  } finally {
    isSaving.value = false
  }
}

async function retryCommand(command: PendingFinancialWrite): Promise<void> {
  pageError.value = ''
  successMessage.value = ''
  isSaving.value = true
  try {
    const receipt = await retryPendingWrite<MutationReceipt>(command.idempotencyKey)
    editingTransaction.value = null
    successMessage.value = `El comando guardado se confirmó (revisión ${receipt.revision}).`
    restorePendingCommands()
    try {
      await loadFirstPage()
    } catch {
      pageError.value = 'El comando se confirmó, pero no se pudo actualizar la lista. Intenta recargarla.'
    }
  } catch (error) {
    if (error instanceof ApiClientError && error.fields.length > 0) {
      serverErrors.value = Object.fromEntries(error.fields.map((field) => [field.field, fieldMessage(field.code)]))
    }
    restorePendingCommands()
    pageError.value = messageFor(error, 'No se pudo confirmar el comando pendiente.')
  } finally {
    isSaving.value = false
  }
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
  // Dismiss only hides this notice. The stored command remains available after reload.
  try {
    dismissPendingWrite(command.idempotencyKey, getPendingStore())
  } catch (error) {
    pageError.value = messageFor(error, 'No fue posible ocultar el aviso.')
  }
  dismissedPendingKeys.value = new Set([...dismissedPendingKeys.value, command.idempotencyKey])
}

function beginEdit(transaction: TransactionView): void {
  editingTransaction.value = transaction
  serverErrors.value = {}
  pageError.value = ''
  successMessage.value = ''
  document.getElementById('transaction-editor')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function cancelEdit(): void {
  // This resets only the local draft. Unresolved writes are never removed here.
  editingTransaction.value = null
  serverErrors.value = {}
}

function beginCreate(): void {
  cancelEdit()
  pageError.value = ''
  successMessage.value = ''
  document.getElementById('transaction-editor')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  document.querySelector<HTMLFormElement>('#transaction-editor form input')?.focus({ preventScroll: true })
}

function hasPendingCommandKey(error: unknown): boolean {
  return typeof error === 'object' && error !== null && 'pendingCommandKey' in error
}

function messageFor(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}

function fieldMessage(code: string): string {
  const messages: Record<string, string> = {
    required_for_opening_date: 'Elige cómo tratar el movimiento respecto del saldo inicial.',
    unsupported: 'Selecciona una opción válida.',
    required: 'Completa este campo.',
    must_be_positive: 'Escribe un monto mayor que cero.',
    same_currency_amounts_must_match: 'Los montos deben coincidir para monedas iguales.',
    must_differ: 'Elige una billetera distinta.',
    required_for_cross_currency: 'Añade evidencia de conversión para esta transferencia.',
    unknown_or_archived: 'La categoría ya no está disponible.',
    must_match_source_expense: 'El reembolso debe seguir la categoría del gasto original.',
    refund_only: 'Solo un reembolso puede vincularse con un gasto.',
    cannot_link_to_self: 'El movimiento no puede vincularse consigo mismo.',
    stale_revision: 'El movimiento cambió. Recarga su versión actual antes de corregirlo.',
  }
  return messages[code] ?? 'Revisa este campo.'
}
</script>

<template>
  <main class="transactions-page" aria-labelledby="transactions-title">
    <header class="page-header">
      <div>
        <p class="eyebrow">Finanzas personales y del hogar</p>
        <h1 id="transactions-title">Movimientos</h1>
        <p class="intro">Registra cada ingreso, gasto, reembolso o transferencia con su fecha y moneda reales.</p>
      </div>
      <button type="button" class="primary-button" @click="beginCreate">Añadir transacción</button>
    </header>

    <p v-if="pageError" class="notice error-notice" role="alert">{{ pageError }}</p>
    <p v-if="successMessage" class="notice success-notice" role="status">{{ successMessage }}</p>

    <section v-if="pendingCommands.length > 0" class="pending-section" aria-labelledby="pending-title">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Resultado pendiente</p>
          <h2 id="pending-title">Comandos que necesitan confirmación</h2>
        </div>
      </div>
      <ul class="pending-list">
        <li
          v-for="command in pendingCommands.filter((item) => !dismissedPendingKeys.has(item.idempotencyKey))"
          :key="command.idempotencyKey"
          class="pending-card"
        >
          <div>
            <strong>{{ command.method === 'POST' ? 'Nuevo movimiento' : 'Corrección de movimiento' }}</strong>
            <p>El resultado anterior no se confirmó. Reintenta el mismo comando con su contenido original.</p>
            <small>{{ new Date(command.createdAt).toLocaleString('es-CL') }}</small>
          </div>
          <div class="pending-actions">
            <button type="button" class="primary-button" :disabled="isSaving" @click="retryCommand(command)">
              {{ isSaving ? 'Reintentando…' : 'Reintentar exactamente' }}
            </button>
            <button type="button" class="text-button" :disabled="isSaving" @click="dismissCommand(command)">
              Ocultar aviso
            </button>
          </div>
        </li>
      </ul>
    </section>

    <section class="workspace-grid" aria-label="Registrar y revisar movimientos">
      <div id="transaction-editor" class="editor-panel">
        <TransactionForm
          :accounts="accounts"
          :categories="categories"
          :expenses="transactions"
          :transaction="editingTransaction"
          :saving="isSaving"
          :server-errors="serverErrors"
          :today="today"
          @submit="submitWrite"
          @cancel="cancelEdit"
        />
      </div>

      <div class="list-panel">
        <TransactionList
          :items="transactions"
          :accounts="accounts"
          :categories="categories"
          :loading="isLoading || !initialized"
          @edit="beginEdit"
        />
        <div v-if="nextCursor" class="pagination-actions">
          <button type="button" class="secondary-button" :disabled="isLoadingMore" @click="loadMore">
            {{ isLoadingMore ? 'Cargando…' : 'Cargar movimientos anteriores' }}
          </button>
        </div>
        <p v-else-if="transactions.length > 0" class="end-of-list">Llegaste al final de los movimientos disponibles.</p>
      </div>
    </section>
  </main>
</template>

<style scoped>
.transactions-page { width: 100%; min-height: 0; margin-inline: auto; padding-block: 8px 24px; color: var(--text); }
.page-header { margin-bottom: 22px; }
.eyebrow { margin: 0 0 10px; color: var(--muted); font-size: 13px; font-weight: 650; letter-spacing: .08em; text-transform: uppercase; }
h1 { margin: 0; font-size: clamp(34px, 5vw, 52px); letter-spacing: -.05em; line-height: 1.05; }
.intro { max-width: 660px; margin: 14px 0 0; color: var(--muted); font-size: 16px; line-height: 1.5; }
.workspace-grid { display: grid; grid-template-columns: minmax(320px, .9fr) minmax(0, 1.1fr); align-items: start; gap: var(--island-gap); margin-top: 20px; }
.editor-panel, .list-panel, .pending-section { border: 1px solid var(--border); min-width: 0; border-radius: var(--island-radius); padding: var(--island-padding); background: var(--surface); box-shadow: var(--shadow-island); }
.list-panel { display: grid; gap: 16px; }
.pending-section { margin-bottom: 18px; border-color: var(--warning-border); background: var(--warning-bg); }
.section-heading h2 { margin: 0; font-size: 21px; }
.pending-list { display: grid; gap: 10px; margin: 14px 0 0; padding: 0; list-style: none; }
.pending-card { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 12px; padding: 13px; border: 1px solid var(--warning-border); border-radius: var(--control-radius); background: var(--surface-raised); }
.pending-card p { margin: 4px 0; color: var(--muted); font-size: 13px; line-height: 1.4; }
.pending-card small { color: var(--muted); }
.pending-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
.notice { border-radius: var(--control-radius); padding: 12px 14px; line-height: 1.45; }
.error-notice { color: var(--danger); background: var(--danger-bg); }
.success-notice { color: var(--success); background: var(--success-bg); }
.primary-button, .secondary-button { min-height: 42px; border-radius: var(--control-radius); padding: 9px 14px; font: inherit; font-weight: 650; cursor: pointer; }
.primary-button { color: var(--on-accent); border: 1px solid var(--accent); background: var(--accent); }
.secondary-button { color: var(--text); border: 1px solid var(--border-strong); background: transparent; }
.text-button { border: 0; padding: 4px 0; color: var(--accent); background: transparent; font: inherit; font-weight: 650; cursor: pointer; text-decoration: underline; text-underline-offset: 2px; }
.pagination-actions { display: flex; justify-content: center; }
.end-of-list { margin: 0; color: var(--muted); text-align: center; font-size: 13px; }
button:disabled { opacity: .6; cursor: not-allowed; }
@media (max-width: 900px) { .workspace-grid { grid-template-columns: 1fr; } }
@media (max-width: 640px) { .transactions-page { width: 100%; padding-top: 28px; }.editor-panel, .list-panel, .pending-section { padding: 15px; } }
</style>
