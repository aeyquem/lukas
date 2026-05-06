<script setup lang="ts">
import type { MonthlyCashflowDto } from "../../api/ledger";
import { formatMoney } from "../../formatters";
import BaseCard from "../base/BaseCard.vue";

defineProps<{
  cashflow: MonthlyCashflowDto | undefined;
}>();
</script>

<template>
  <section class="summary-grid" aria-label="Resumen mensual">
    <BaseCard class="metric">
      <span>Ingresos</span>
      <strong>{{
        cashflow
          ? formatMoney(cashflow.incomeTotal, cashflow.currency)
          : formatMoney("0", "CLP")
      }}</strong>
    </BaseCard>
    <BaseCard class="metric">
      <span>Gastos</span>
      <strong>{{
        cashflow
          ? formatMoney(cashflow.expenseTotal, cashflow.currency)
          : formatMoney("0", "CLP")
      }}</strong>
    </BaseCard>
    <BaseCard class="metric">
      <span>Flujo neto</span>
      <strong>{{
        cashflow
          ? formatMoney(cashflow.netCashflow, cashflow.currency)
          : formatMoney("0", "CLP")
      }}</strong>
    </BaseCard>
  </section>
</template>

<style scoped>
.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.metric {
  min-inline-size: 0;
}

.metric :deep(.base-card__body) {
  display: grid;
  gap: var(--space-1);
}

.metric span {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.metric strong {
  color: var(--color-foreground);
  font-family: var(--font-mono);
  font-size: clamp(1.05rem, 2vw, 1.45rem);
}

@media (max-width: 680px) {
  .summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
