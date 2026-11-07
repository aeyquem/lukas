<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAccountListQuery } from '../features/accounts/accounts-feature'
import TransactionForm from '../features/transactions/components/transaction-form.vue'
import TransactionList from '../features/transactions/components/transaction-list.vue'
import type { PendingFinancialWrite } from '../shared/api/pending-commands'
import type { TransactionView } from '../features/transactions/api/contracts'
import { useTransactionCategoriesQuery } from '../features/transactions/api/queries'
import { useTransactionPages } from '../features/transactions/composables/use-transaction-pages'
import { useTransactionCommands } from '../features/transactions/composables/use-transaction-commands'

const accountQuery = useAccountListQuery()
const accounts = computed(() => accountQuery.data.value ?? [])
const categoriesQuery = useTransactionCategoriesQuery()
const categoriesLoaded = ref(false)
const categories = computed(() => categoriesLoaded.value ? categoriesQuery.data.value ?? [] : [])
const transactionPages = useTransactionPages()
const transactions = transactionPages.items
const nextCursor = transactionPages.nextCursor
const isLoadingMore = transactionPages.isLoadingMore
const {
  visiblePendingCommands: pendingCommands,
  serverErrors,
  pageError,
  successMessage,
  isSaving,
  restorePendingCommands,
  dismissCommand,
  submitWrite: submitTransactionWrite,
  retryCommand: retryTransactionCommand,
} = useTransactionCommands()
const editingTransaction = ref<TransactionView | null>(null)
const isLoading = ref(false)
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

onMounted(() => {
  restorePendingCommands()
  void loadInitialData()
})

async function loadInitialData(): Promise<void> {
  isLoading.value = true
  pageError.value = ''
  try {
    const [, categoryResult] = await Promise.all([
      accountQuery.refresh(true),
      categoriesQuery.refetch(true),
    ])
    if (categoryResult.status !== 'success') {
      throw categoryResult.error ?? new Error('No fue posible cargar las categorías de movimientos.')
    }
    categoriesLoaded.value = true
    await loadFirstPage()
    initialized.value = true
  } catch (error) {
    pageError.value = messageFor(error, 'No fue posible cargar los movimientos.')
  } finally {
    isLoading.value = false
  }
}

async function loadFirstPage(): Promise<void> {
  await transactionPages.loadFirstPage()
}

async function loadMore(): Promise<void> {
  pageError.value = ''
  try { await transactionPages.loadNextPage() }
  catch (error) { pageError.value = messageFor(error, 'No fue posible cargar la página siguiente.') }
}

async function retryCommand(command: PendingFinancialWrite): Promise<void> {
  await retryTransactionCommand(command, loadFirstPage, () => { editingTransaction.value = null })
}

async function submitWrite(command: { method: 'POST' | 'PUT'; route: string; body: string }): Promise<void> {
  await submitTransactionWrite(command, loadFirstPage, () => { editingTransaction.value = null })
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

function messageFor(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
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
          v-for="command in pendingCommands"
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
.list-panel { display: grid; grid-template-columns: minmax(0, 1fr); gap: 16px; }
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
