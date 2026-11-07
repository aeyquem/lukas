<script setup lang="ts">
import { computed, ref } from 'vue'
import { formatBusinessDate } from '../../../shared/formatting/business-date'
import { formatMoney } from '../../../shared/formatting/money'
import type { AccountView, TransactionCategory, TransactionView } from '../transaction-model'

const props = defineProps<{
  items: TransactionView[]
  accounts: AccountView[]
  categories: TransactionCategory[]
  loading: boolean
}>()

const emit = defineEmits<{ edit: [transaction: TransactionView] }>()
const search = ref('')
const sortField = ref<'date' | 'payee' | 'category' | 'note'>('date')
const sortDescending = ref(true)

const kindLabels: Record<TransactionView['kind'], string> = {
  income: 'Ingreso', expense: 'Gasto', refund: 'Reembolso', transfer: 'Transferencia',
}

function categoryName(id: string | null): string {
  return id ? props.categories.find((category) => category.id === id)?.name ?? 'Categoría archivada' : 'Sin categoría'
}

function accountName(walletId: string): string {
  for (const account of props.accounts) {
    if (account.wallets.some((wallet) => wallet.id === walletId)) return account.name
  }
  return 'Cuenta'
}

function amounts(transaction: TransactionView, incoming: boolean): string[] {
  return transaction.effects.flatMap((effect) => {
    const isIncoming = !effect.amount.startsWith('-')
    if (isIncoming !== incoming) return []
    const value = effect.amount.startsWith('-') ? effect.amount.slice(1) : effect.amount
    return [`${formatMoney(value, effect.currency)} · ${accountName(effect.wallet_id)}`]
  })
}

const matchingItems = computed(() => {
  const needle = search.value.trim().toLocaleLowerCase('es-CL')
  const filtered = props.items.filter((transaction) => {
    if (!needle) return true
    return [
      transaction.business_date,
      kindLabels[transaction.kind],
      transaction.payee ?? '',
      categoryName(transaction.category_id),
      transaction.note ?? '',
      ...transaction.effects.map((effect) => `${effect.currency} ${effect.amount} ${accountName(effect.wallet_id)}`),
    ].join(' ').toLocaleLowerCase('es-CL').includes(needle)
  })
  return filtered.sort((a, b) => {
    const left = sortField.value === 'date' ? a.business_date : sortField.value === 'payee' ? (a.payee ?? kindLabels[a.kind]) : sortField.value === 'category' ? categoryName(a.category_id) : (a.note ?? '')
    const right = sortField.value === 'date' ? b.business_date : sortField.value === 'payee' ? (b.payee ?? kindLabels[b.kind]) : sortField.value === 'category' ? categoryName(b.category_id) : (b.note ?? '')
    const comparison = left.localeCompare(right, 'es-CL', { sensitivity: 'base' })
    return sortDescending.value ? -comparison : comparison
  })
})

function toggleSort(field: typeof sortField.value): void {
  if (sortField.value === field) sortDescending.value = !sortDescending.value
  else {
    sortField.value = field
    sortDescending.value = field === 'date'
  }
}

function ariaSort(field: typeof sortField.value): 'ascending' | 'descending' | 'none' {
  return sortField.value !== field ? 'none' : sortDescending.value ? 'descending' : 'ascending'
}
</script>

<template>
  <section class="activity-list" aria-labelledby="activity-list-title">
    <header class="list-heading">
      <div><p class="eyebrow">Registro financiero</p><h2 id="activity-list-title">Transacciones</h2></div>
      <span v-if="!loading" class="count">{{ matchingItems.length }} / {{ items.length }} cargadas</span>
    </header>
    <label class="search-field" for="transaction-search">Buscar entre las transacciones cargadas
      <input id="transaction-search" v-model="search" type="search" autocomplete="off" placeholder="Payee, categoría, nota…">
    </label>
    <p class="table-scope">La búsqueda y el orden se aplican a las páginas ya cargadas.</p>
    <p v-if="loading" class="muted" role="status">Cargando movimientos…</p>
    <p v-else-if="items.length === 0" class="empty-state">No hay movimientos en esta página.</p>
    <p v-else-if="matchingItems.length === 0" class="empty-state">No hay transacciones cargadas que coincidan con la búsqueda.</p>
    <div v-else class="table-scroll">
      <table>
        <thead><tr>
          <th scope="col" :aria-sort="ariaSort('date')"><button type="button" @click="toggleSort('date')">Fecha <span aria-hidden="true">↕</span></button></th>
          <th scope="col" :aria-sort="ariaSort('payee')"><button type="button" @click="toggleSort('payee')">Payee <span aria-hidden="true">↕</span></button></th>
          <th scope="col" :aria-sort="ariaSort('category')"><button type="button" @click="toggleSort('category')">Categoría <span aria-hidden="true">↕</span></button></th>
          <th scope="col">Tags</th><th scope="col" :aria-sort="ariaSort('note')"><button type="button" @click="toggleSort('note')">Notas <span aria-hidden="true">↕</span></button></th>
          <th scope="col">Entra</th><th scope="col">Sale</th><th scope="col">Conciliación</th><th scope="col">Acción</th>
        </tr></thead>
        <tbody>
          <tr v-for="transaction in matchingItems" :key="transaction.id">
            <td class="numeric">{{ formatBusinessDate(transaction.business_date) }}<small class="kind-label">{{ kindLabels[transaction.kind] }}</small></td>
            <td>{{ transaction.payee || '—' }}</td><td>{{ categoryName(transaction.category_id) }}</td>
            <td><span class="unavailable-field">No disponible</span></td>
            <td class="note-cell">{{ transaction.note || '—' }}</td>
            <td><span v-for="amount in amounts(transaction, true)" :key="amount" class="amount-line numeric">{{ amount }}</span><span v-if="amounts(transaction, true).length === 0">—</span></td>
            <td><span v-for="amount in amounts(transaction, false)" :key="amount" class="amount-line numeric">{{ amount }}</span><span v-if="amounts(transaction, false).length === 0">—</span></td>
            <td><span class="unavailable-field">No disponible</span></td>
            <td><button type="button" class="edit-button" :aria-label="`Editar ${kindLabels[transaction.kind].toLowerCase()} del ${transaction.business_date}`" @click="emit('edit', transaction)">Editar</button></td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.activity-list { display: grid; grid-template-columns: minmax(0, 1fr); gap: 12px; min-width: 0; }
.list-heading { display: flex; align-items: center; justify-content: space-between; gap: 14px; }
.list-heading h2 { margin: 0; font-size: 24px; }
.eyebrow { margin: 0 0 6px; color: var(--muted); font-size: 12px; font-weight: 650; letter-spacing: .08em; text-transform: uppercase; }
.count, .kind-label, .unavailable-field, .table-scope { color: var(--muted); font-size: 11px; }
.search-field { display: grid; gap: 6px; font-size: 12px; font-weight: 650; }
.search-field input { min-height: 42px; padding: 8px 12px; border: 1px solid var(--border-strong); border-radius: var(--control-radius); color: var(--text); background: var(--surface-raised); font: inherit; }
.table-scope { margin: -6px 0 0; }
.table-scroll { overflow-x: auto; min-width: 0; }
table { width: 100%; border-collapse: collapse; font-size: 12px; }
th, td { padding: 9px 8px; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; }
th { white-space: nowrap; }
th button { padding: 3px 0; border: 0; color: inherit; background: none; font: inherit; font-weight: 700; cursor: pointer; }
th button:hover { color: var(--accent); }
td { min-width: 75px; }
.kind-label, .amount-line { display: block; margin-top: 3px; }
.note-cell { min-width: 120px; max-width: 240px; overflow-wrap: anywhere; }
.edit-button { min-height: 34px; padding: 5px 9px; border: 1px solid var(--border-strong); border-radius: var(--control-radius); color: var(--accent); background: var(--surface-soft); font: inherit; cursor: pointer; }
.empty-state, .muted { margin: 0; color: var(--muted); }
@media (max-width: 640px) { .list-heading { align-items: start; flex-direction: column; } }
</style>
