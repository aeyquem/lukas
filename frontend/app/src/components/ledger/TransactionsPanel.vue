<script setup lang="ts">
import type { TransactionDto } from "../../api/ledger";
import { formatDate, formatMoney } from "../../formatters";
import { transactionKindLabel } from "../../ledger/labels";
import BaseAlert from "../base/BaseAlert.vue";
import BaseBadge from "../base/BaseBadge.vue";
import BaseCard from "../base/BaseCard.vue";

const props = defineProps<{
  hasLoadError: boolean;
  transactions: TransactionDto[];
}>();
</script>

<template>
  <BaseCard class="panel transactions-panel">
    <template #header>
      <div class="panel-heading">
        <h2>Movimientos del periodo</h2>
        <BaseBadge>{{ props.transactions.length }}</BaseBadge>
      </div>
    </template>

    <BaseAlert v-if="props.hasLoadError" class="form-alert" tone="danger">
      No se pudieron cargar los movimientos.
    </BaseAlert>

    <ul class="transaction-list">
      <li v-for="transaction in props.transactions" :key="transaction.id">
        <div>
          <strong>{{ transaction.description }}</strong>
          <span>
            {{ transactionKindLabel(transaction.kind) }} ·
            {{ formatDate(transaction.transactionDate) }}
          </span>
        </div>
        <b>{{ formatMoney(transaction.amount, transaction.currency) }}</b>
      </li>
    </ul>
  </BaseCard>
</template>

<style scoped>
.transactions-panel {
  grid-column: 1 / -1;
}

.panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-block-end: 0;
}

.panel-heading h2 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1rem;
}

.form-alert {
  margin-block-end: var(--space-4);
}

.transaction-list {
  display: grid;
  gap: var(--space-2);
  list-style: none;
  margin: 0;
  padding: 0;
}

.transaction-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  border-block-start: 1px solid var(--color-border);
  padding-block: var(--space-3);
}

.transaction-list div {
  display: grid;
  gap: var(--space-1);
  min-inline-size: 0;
}

.transaction-list span {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.transaction-list strong {
  overflow-wrap: anywhere;
}

.transaction-list b {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  white-space: nowrap;
}

@media (max-width: 680px) {
  .transaction-list li {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>
