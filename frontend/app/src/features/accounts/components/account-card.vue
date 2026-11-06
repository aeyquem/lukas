<script setup lang="ts">
import { formatMoney } from '../../shared/formatting/money'

interface WalletSummary {
  id: string
  currency: string
  balance: string
}

interface AccountSummary {
  id: string
  name: string
  account_type: string
  base_currency: string
  budget_eligible: boolean
  wallets: WalletSummary[]
}

defineProps<{
  account: AccountSummary
  selected: boolean
}>()

const emit = defineEmits<{
  select: []
}>()

const accountTypeLabels: Record<string, string> = {
  bank: 'Banco',
  cash: 'Efectivo',
  savings: 'Ahorro',
  card: 'Tarjeta',
  investment: 'Inversión',
}
const accountTypeIcons: Record<string, string> = {
  bank: '▤', cash: '◇', savings: '▣', card: '▭', investment: '↗',
}
</script>

<template>
  <button
    type="button"
    class="account-card"
    :aria-current="selected ? 'true' : undefined"
    @click="emit('select')"
  >
    <span class="account-card-heading">
      <span class="account-icon" aria-hidden="true">{{ accountTypeIcons[account.account_type] ?? '◉' }}</span>
      <strong>{{ account.name }}</strong>
      <span class="account-kind">{{ accountTypeLabels[account.account_type] ?? account.account_type }}</span>
    </span>
    <span class="account-meta">Base {{ account.base_currency }} · {{ account.budget_eligible ? 'Participa en presupuesto' : 'Solo seguimiento' }}</span>
    <span class="balances">
      <span v-for="wallet in account.wallets" :key="wallet.id" class="balance-line">
        <span>{{ wallet.currency }}</span>
        <strong class="numeric">{{ formatMoney(wallet.balance, wallet.currency) }}</strong>
      </span>
    </span>
  </button>
</template>

<style scoped>
.account-card {
  display: grid;
  width: 100%;
  gap: 8px;
  padding: 15px;
  border: 1px solid var(--border);
  border-radius: var(--control-radius);
  color: inherit;
  background: var(--surface-raised);
  text-align: left;
  cursor: pointer;
}

.account-card:hover,
.account-card[aria-current='true'] {
  border-color: var(--border-strong);
  background: var(--surface-soft);
}

.account-card strong {
  font-size: 15px;
}
.account-icon { display: grid; width: 32px; height: 32px; flex: 0 0 32px; place-items: center; border-radius: 9px; color: var(--accent); background: var(--accent-soft); font-size: 18px; }

.account-card-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.account-kind {
  padding: 4px 8px;
  border-radius: 999px;
  color: var(--muted);
  background: var(--accent-soft);
  font-size: 11px;
  font-weight: 700;
}

.account-meta {
  color: var(--muted);
  font-size: 12px;
}

.balances {
  display: grid;
  gap: 4px;
  padding-top: 7px;
  border-top: 1px solid var(--border);
}

.balance-line {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}
</style>
