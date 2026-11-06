<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import type { TransactionView } from '../components/activity/transaction-model'
import { budgetRoute, currentPeriodInSantiago, type BudgetSnapshot, type BudgetValue } from '../components/budget/budget-model'
import { useAccountListQuery } from '../features/accounts/accounts-feature'
import { onFinanceInvalidated } from '../shared/api/finance-invalidation'
import { apiRequest } from '../shared/api/client'
import { formatBusinessDate } from '../shared/formatting/business-date'
import { formatMoney } from '../shared/formatting/money'

interface TransactionPage { items: TransactionView[]; next: { before_date: string; before_id: string } | null }

const route = useRoute()
const period = computed(() => typeof route.query.dashboardPeriod === 'string' && /^\d{4}-(0[1-9]|1[0-2])$/.test(route.query.dashboardPeriod) ? route.query.dashboardPeriod : currentPeriodInSantiago())
const accountQuery = useAccountListQuery()
const accounts = computed(() => accountQuery.data.value ?? null)
const budget = ref<BudgetSnapshot | null>(null)
const transactions = ref<TransactionView[] | null>(null)
const accountError = computed(() => accountQuery.error.value ? 'No fue posible cargar los saldos.' : '')
const budgetError = ref('')
const transactionError = ref('')
const loading = ref(false)
let latestRequest = 0

const moneyText = (value: BudgetValue | undefined) => {
  if (!value) return 'No disponible'
  if (value.state === 'complete') return formatMoney(value.amount ?? '0', 'CLP')
  if (value.state === 'incomplete') return `Incompleto · subtotal conocido ${formatMoney(value.known_subtotal ?? '0', 'CLP')}`
  return 'No disponible'
}
const recentTransactions = computed(() => (transactions.value ?? []).slice(0, 6))
const transactionLabel = (transaction: TransactionView) => transaction.payee || ({ income: 'Ingreso', expense: 'Gasto', refund: 'Reembolso', transfer: 'Transferencia' }[transaction.kind])

async function loadDashboard(): Promise<void> {
  const requestId = ++latestRequest
  loading.value = true
  budgetError.value = ''; transactionError.value = ''
  const results = await Promise.allSettled([
    apiRequest<BudgetSnapshot>(budgetRoute(period.value)),
    apiRequest<TransactionPage>('transactions?page_size=20'),
  ])
  if (requestId !== latestRequest) return
  const [budgetResult, transactionResult] = results
  budget.value = budgetResult.status === 'fulfilled' ? budgetResult.value : null
  transactions.value = transactionResult.status === 'fulfilled' ? transactionResult.value.items : null
  budgetError.value = budgetResult.status === 'rejected' ? 'No fue posible cargar el presupuesto de este período.' : ''
  transactionError.value = transactionResult.status === 'rejected' ? 'No fue posible cargar la actividad reciente.' : ''
  loading.value = false
}

onFinanceInvalidated(loadDashboard)
onMounted(() => { void loadDashboard() })
function refreshDashboard() { void accountQuery.refetch(); void loadDashboard() }
watch(period, () => {
  budget.value = null
  void loadDashboard()
})
</script>

<template>
  <main class="dashboard-page" aria-labelledby="home-title">
    <header class="dashboard-heading">
      <div><p class="eyebrow">Tu espacio, a tu ritmo</p><h1 id="home-title">Panel financiero</h1><p>Resumen de {{ period }} · CLP para el presupuesto.</p></div>
      <button type="button" class="refresh-button" :disabled="loading" @click="refreshDashboard">{{ loading ? 'Actualizando…' : 'Actualizar resumen' }}</button>
    </header>

    <section class="dashboard-grid" aria-label="Resumen financiero">
      <article class="dashboard-panel">
        <header><div><p class="eyebrow">Tus cuentas</p><h2>Saldos por cuenta</h2></div><RouterLink :to="{ name: 'accounts' }">Ver cuentas <span aria-hidden="true">↗</span></RouterLink></header>
        <p v-if="accountError" class="section-error" role="alert">{{ accountError }}</p>
        <p v-else-if="accounts === null" role="status">Cargando saldos…</p>
        <p v-else-if="accounts.length === 0" class="empty-state">Todavía no hay cuentas registradas.</p>
        <ul v-else class="account-summary-list">
          <li v-for="account in accounts" :key="account.id">
            <strong>{{ account.name }}</strong>
            <span v-for="wallet in account.wallets" :key="wallet.id" class="numeric">{{ formatMoney(wallet.balance, wallet.currency) }}</span>
          </li>
        </ul>
      </article>

      <article class="dashboard-panel budget-summary">
        <header><div><p class="eyebrow">Planificación</p><h2>Presupuesto del período</h2></div><RouterLink :to="{ name: 'budget' }">Abrir presupuesto <span aria-hidden="true">↗</span></RouterLink></header>
        <p v-if="budgetError" class="section-error" role="alert">{{ budgetError }}</p>
        <p v-else-if="budget === null" role="status">Cargando presupuesto…</p>
        <dl v-else class="budget-summary-list">
          <div><dt>Disponible para asignar</dt><dd class="numeric">{{ moneyText(budget.ready_to_assign) }}</dd></div>
          <div><dt>Financiamiento</dt><dd class="numeric">{{ moneyText(budget.funding) }}</dd></div>
          <div><dt>Actividad del período</dt><dd class="numeric">{{ moneyText(budget.activity) }}</dd></div>
          <div><dt>Gastos sin categoría</dt><dd class="numeric">{{ moneyText(budget.uncategorized_activity) }}</dd></div>
          <div><dt>Ingresos del período</dt><dd>No disponible en el resumen del presupuesto</dd></div>
        </dl>
      </article>

      <article class="dashboard-panel recent-summary">
        <header><div><p class="eyebrow">Registro financiero</p><h2>Movimientos recientes</h2></div><RouterLink :to="{ name: 'transactions' }">Ver transacciones <span aria-hidden="true">↗</span></RouterLink></header>
        <p v-if="transactionError" class="section-error" role="alert">{{ transactionError }}</p>
        <p v-else-if="transactions === null" role="status">Cargando movimientos…</p>
        <p v-else-if="recentTransactions.length === 0" class="empty-state">Todavía no hay movimientos.</p>
        <ul v-else class="recent-list">
          <li v-for="transaction in recentTransactions" :key="transaction.id">
            <div><strong>{{ transactionLabel(transaction) }}</strong><small>{{ formatBusinessDate(transaction.business_date) }}</small></div>
            <span v-for="effect in transaction.effects" :key="`${effect.wallet_id}-${effect.role}`" class="numeric">{{ formatMoney(effect.amount, effect.currency) }}</span>
          </li>
        </ul>
      </article>
    </section>
  </main>
</template>

<style scoped>
.dashboard-page { display: grid; gap: var(--island-gap); padding-block: 8px 24px; }
.dashboard-heading, .dashboard-panel > header { display: flex; align-items: end; justify-content: space-between; gap: 18px; }
.dashboard-heading h1 { margin: 0; font-size: clamp(32px, 4vw, 48px); letter-spacing: -.05em; }
.dashboard-heading p:not(.eyebrow) { color: var(--muted); }
.eyebrow { margin: 0 0 6px; color: var(--muted); font-size: 11px; letter-spacing: .08em; text-transform: uppercase; }
.dashboard-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--island-gap); align-items: start; }
.dashboard-panel { min-width: 0; padding: var(--island-padding); border: 1px solid var(--border); border-radius: var(--island-radius); background: var(--surface); box-shadow: var(--shadow-island); }
.dashboard-panel > header { align-items: start; margin-bottom: 18px; }
.dashboard-panel h2 { margin: 0; font-size: 20px; }
.dashboard-panel > header > a { color: var(--accent); font-size: 12px; font-weight: 650; text-decoration: none; white-space: nowrap; }
.dashboard-panel > header > a:hover { text-decoration: underline; }
.account-summary-list, .recent-list { display: grid; gap: 0; margin: 0; padding: 0; list-style: none; }
.account-summary-list li, .recent-list li { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 8px 16px; padding: 12px 0; border-top: 1px solid var(--border); }
.account-summary-list strong { min-width: 110px; flex: 1; }
.recent-list li > div { display: grid; gap: 3px; min-width: 130px; flex: 1; }
.recent-list small { color: var(--muted); }
.budget-summary-list { display: grid; gap: 0; margin: 0; }
.budget-summary-list div { display: flex; justify-content: space-between; gap: 16px; padding: 12px 0; border-top: 1px solid var(--border); }
.budget-summary-list dt { color: var(--muted); }
.budget-summary-list dd { margin: 0; text-align: right; }
.empty-state, .section-error { margin: 0; padding: 14px; border-radius: var(--control-radius); background: var(--surface-soft); color: var(--muted); }
.section-error { color: var(--danger); background: var(--danger-bg); }
.refresh-button { min-height: 42px; padding: 8px 14px; border: 1px solid var(--border-strong); border-radius: var(--control-radius); color: var(--accent); background: var(--surface-soft); font: inherit; cursor: pointer; }
.refresh-button:disabled { opacity: .6; cursor: wait; }
@media (max-width: 760px) { .dashboard-grid { grid-template-columns: 1fr; }.dashboard-heading { align-items: start; flex-direction: column; }.dashboard-panel > header { flex-direction: column; } }
</style>
