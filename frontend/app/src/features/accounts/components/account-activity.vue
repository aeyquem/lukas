<script setup lang="ts">
import { computed } from 'vue'
import { useAccountListQuery } from '../api/queries'
import { useAccountActivityQuery } from '../../transactions/api/queries'
import { formatMoney } from '../../../shared/formatting/money'
import { formatBusinessDate } from '../../../shared/formatting/business-date'
const props = defineProps<{
  accountId: string
}>()
const query = useAccountActivityQuery(() => props.accountId)
const list = useAccountListQuery()
const accountTransactions = computed(() => query.data.value?.items ?? [])
const accountTransactionsLoading = query.isLoading
const accountTransactionsError = computed(() => query.error.value?.message ?? '')
function accountNameForWallet(walletId: string) { return list.data.value?.find(a => a.wallets.some(w => w.id === walletId))?.name ?? 'Cuenta'; }
</script>
<template>
<section class="account-activity" aria-labelledby="account-activity-title">
  <div class="section-heading">
    <div>
      <p class="eyebrow">Registro de esta cuenta</p>
      <h3 id="account-activity-title">Transacciones</h3>
    </div>
    <button type="button" class="text-button" :disabled="accountTransactionsLoading" @click="query.refetch()">Actualizar actividad</button>
  </div>
  <p v-if="accountTransactionsLoading" role="status" class="muted">Cargando actividad…</p>
  <p v-else-if="accountTransactionsError" role="alert" class="notice error-notice">{{ accountTransactionsError }}</p>
  <p v-else-if="accountTransactions.length === 0" class="empty-state">Esta cuenta todavía no tiene transacciones.</p>
  <div v-else class="account-activity-table-wrap">
    <table class="account-activity-table">
      <thead><tr><th scope="col">Fecha</th><th scope="col">Tipo</th><th scope="col">Payee</th><th scope="col">Efectos</th></tr></thead>
      <tbody>
        <tr v-for="transaction in accountTransactions" :key="transaction.id">
          <td class="numeric">{{ formatBusinessDate(transaction.business_date) }}</td>
          <td>{{ transaction.kind === 'income' ? 'Ingreso' : transaction.kind === 'expense' ? 'Gasto' : transaction.kind === 'refund' ? 'Reembolso' : 'Transferencia' }}</td>
          <td>{{ transaction.payee || '—' }}<small v-if="transaction.note" class="activity-note">{{ transaction.note }}</small></td>
          <td><span v-for="effect in transaction.effects" :key="`${effect.wallet_id}-${effect.role}`" class="activity-effect"><small>{{ effect.role === 'source' ? 'Sale de' : effect.role === 'destination' ? 'Llega a' : 'Efecto en' }} {{ accountNameForWallet(effect.wallet_id) }} · {{ effect.currency }}</small><strong class="numeric">{{ formatMoney(effect.amount, effect.currency) }}</strong></span></td>
        </tr>
      </tbody>
    </table>
    <p class="help-copy">Se muestran hasta 50 transacciones recientes de esta cuenta.</p>
  </div>
</section>
</template>
<style scoped>
.section-heading {
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

.help-copy,
.muted,
.empty-state {
  color: var(--muted);
  line-height: 1.5;
}

.eyebrow {
  margin-bottom: 8px;
}

.section-heading {
  margin-bottom: 18px;
}

.help-copy {
  margin-bottom: 0;
  font-size: 12px;
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

.account-activity { margin-top: 24px; padding: 18px; border: 1px solid var(--border); border-radius: var(--control-radius); background: var(--surface-soft); }

.account-activity h3 { margin: 0; }

.account-activity-table-wrap { min-width: 0; overflow-x: auto; }

.account-activity-table { width: 100%; border-collapse: collapse; font-size: 12px; }

.account-activity-table th,
.account-activity-table td { padding: 9px 8px; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; }

.activity-effect { display: block; white-space: nowrap; }

.activity-note { display: block; max-width: 230px; margin-top: 4px; color: var(--muted); overflow-wrap: anywhere; }

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

.empty-state {
  margin-bottom: 0;
  font-size: 13px;
}
</style>
