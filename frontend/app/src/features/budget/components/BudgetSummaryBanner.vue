<script setup lang="ts">
import { formatMoney } from "@/common/formatters";

defineProps<{
  amount: string;
  currency: string;
  tone: "positive" | "negative";
}>();
</script>

<template>
  <div class="zero-budget-banner">
    <div>
      <p class="eyebrow">Presupuesto base cero</p>
      <strong>Ingresos por asignar</strong>
    </div>
    <span class="zero-budget-banner__amount" :data-tone="tone">
      {{ formatMoney(amount, currency) }}
    </span>
  </div>
</template>

<style scoped>
.zero-budget-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--lu-space-4);
  border: 1px solid var(--lu-color-border);
  border-radius: var(--lu-radius-md);
  background: var(--lu-color-surface-muted);
  margin-block-end: var(--lu-space-4);
  padding: var(--lu-space-4);
}

.zero-budget-banner .eyebrow {
  margin: 0 0 var(--lu-space-1);
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.zero-budget-banner strong {
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-md);
}

.zero-budget-banner__amount {
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-2xl);
  font-weight: 900;
  white-space: nowrap;
}

.zero-budget-banner__amount[data-tone="negative"] {
  color: var(--lu-color-danger);
}
</style>
