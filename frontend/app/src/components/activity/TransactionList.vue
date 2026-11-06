<script setup lang="ts">
import { formatBusinessDate } from '../../shared/formatting/business-date'
import { formatMoney } from '../../shared/formatting/money'
import type { AccountView, TransactionCategory, TransactionView } from './transaction-model'

defineProps<{
  items: TransactionView[]
  accounts: AccountView[]
  categories: TransactionCategory[]
  loading: boolean
}>()

const emit = defineEmits<{ edit: [transaction: TransactionView] }>()

const kindLabels: Record<TransactionView['kind'], string> = {
  income: 'Ingreso',
  expense: 'Gasto',
  refund: 'Reembolso',
  transfer: 'Transferencia',
}

function categoryName(id: string | null, categories: TransactionCategory[]): string | undefined {
  return id ? categories.find((category) => category.id === id)?.name : undefined
}

function walletLabel(id: string, accounts: AccountView[]): string {
  for (const account of accounts) {
    const wallet = account.wallets.find((item) => item.id === id)
    if (wallet) return `${account.name} · ${wallet.currency}`
  }
  return 'Billetera'
}
</script>

<template>
  <section class="activity-list" aria-labelledby="activity-list-title">
    <header class="list-heading">
      <div>
        <p class="eyebrow">Registro financiero</p>
        <h2 id="activity-list-title">Movimientos</h2>
      </div>
      <span v-if="!loading" class="count">{{ items.length }}</span>
    </header>

    <p v-if="loading" class="muted" role="status">Cargando movimientos…</p>
    <p v-else-if="items.length === 0" class="empty-state">No hay movimientos en esta página.</p>
    <ol v-else class="transaction-items">
      <li v-for="transaction in items" :key="transaction.id" :data-transaction-id="transaction.id">
        <article class="transaction-card">
          <div class="transaction-main">
            <div class="transaction-title-row">
              <h3>{{ transaction.payee || kindLabels[transaction.kind] }}</h3>
              <span class="kind-pill">{{ kindLabels[transaction.kind] }}</span>
            </div>
            <p class="transaction-meta">
              {{ formatBusinessDate(transaction.business_date) }}
              <span v-if="categoryName(transaction.category_id, categories)"> · {{ categoryName(transaction.category_id, categories) }}</span>
              <span v-else-if="transaction.kind === 'expense' || transaction.kind === 'refund'"> · Sin categoría</span>
            </p>
            <p v-if="transaction.note" class="transaction-note">{{ transaction.note }}</p>
            <p v-if="transaction.source_expense_id" class="transaction-link">
              Reembolso vinculado al gasto {{ transaction.source_expense_id.slice(0, 8) }}
            </p>
            <p v-if="transaction.kind === 'expense' && transaction.confirmed_clp_amount" class="transaction-meta">
              Valor confirmado: {{ formatMoney(transaction.confirmed_clp_amount, 'CLP') }}
            </p>
          </div>
          <div class="transaction-effects" aria-label="Efectos por billetera">
            <p v-for="effect in transaction.effects" :key="`${effect.wallet_id}-${effect.role}`">
              <span>{{ effect.role === 'source' ? 'Sale de' : effect.role === 'destination' ? 'Llega a' : 'Billetera' }} {{ walletLabel(effect.wallet_id, accounts) }}</span>
              <strong>{{ formatMoney(effect.amount, effect.currency) }}</strong>
              <small v-if="effect.opening_treatment">
                {{ effect.opening_treatment === 'apply_effect' ? 'Incluido además del saldo inicial' : 'Saldo inicial conservado mediante compensación' }}
              </small>
            </p>
          </div>
          <div class="transaction-actions">
            <span class="revision">Revisión {{ transaction.revision }}</span>
            <button
              type="button"
              class="text-button"
              :aria-label="`Corregir ${kindLabels[transaction.kind].toLowerCase()} del ${transaction.business_date}`"
              @click="emit('edit', transaction)"
            >
              Corregir
            </button>
          </div>
        </article>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.activity-list { display: grid; gap: 14px; }
.list-heading { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.list-heading h2 { margin: 0; font-size: 24px; }
.eyebrow { margin: 0 0 6px; color: #52675b; font-size: 12px; font-weight: 650; letter-spacing: .08em; text-transform: uppercase; }
.count, .kind-pill { border-radius: 999px; padding: 4px 9px; color: #415348; background: #e8eee9; font-size: 12px; font-weight: 650; }
.transaction-items { display: grid; gap: 10px; margin: 0; padding: 0; list-style: none; }
.transaction-card { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 8px 18px; padding: 16px; border: 1px solid #dce1db; border-radius: 12px; background: #fff; }
.transaction-title-row { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
.transaction-title-row h3 { margin: 0; font-size: 16px; }
.transaction-meta, .transaction-note, .transaction-link { margin: 5px 0 0; color: #626b63; font-size: 13px; line-height: 1.45; overflow-wrap: anywhere; }
.transaction-link { color: #52675b; }
.transaction-effects { display: grid; gap: 6px; align-content: center; text-align: right; }
.transaction-effects p { display: grid; gap: 2px; margin: 0; font-size: 12px; color: #626b63; }
.transaction-effects strong { color: #272b27; font-size: 14px; font-weight: 650; }
.transaction-actions { grid-column: 1 / -1; display: flex; justify-content: space-between; align-items: center; gap: 10px; border-top: 1px solid #edf0ec; padding-top: 8px; }
.revision { color: #737a73; font-size: 12px; }
.text-button { border: 0; padding: 4px 0; color: #286449; background: transparent; font: inherit; font-weight: 650; cursor: pointer; text-decoration: underline; text-underline-offset: 2px; }
.empty-state, .muted { margin: 0; color: #626b63; }
@media (max-width: 640px) { .transaction-card { grid-template-columns: 1fr; }.transaction-effects { text-align: left; }.transaction-effects p { display: flex; gap: 7px; } }
</style>
