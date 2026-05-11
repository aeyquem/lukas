<script setup lang="ts">
import { useQuery } from "@pinia/colada";
import { computed } from "vue";

import { getMonthlyTagReport } from "@/common/api/ledger";
import BaseAlert from "@/common/components/base/BaseAlert.vue";
import BaseCard from "@/common/components/base/BaseCard.vue";
import { formatMoney } from "@/common/formatters";

const props = defineProps<{
  period: string;
}>();

const tagReportQuery = useQuery({
  key: () => ["tag-report", props.period],
  query: () => getMonthlyTagReport(props.period),
});

const report = computed(() => tagReportQuery.data.value);
const tagRows = computed(() => report.value?.tags ?? []);
</script>

<template>
  <BaseCard class="reports-panel">
    <template #header>
      <div class="reports-heading">
        <div>
          <h2 id="reports-heading">Reportes</h2>
          <span>Gasto por etiqueta</span>
        </div>
      </div>
    </template>

    <section aria-labelledby="reports-heading">
      <BaseAlert
        v-if="tagReportQuery.error.value"
        class="reports-alert"
        tone="danger"
      >
        No se pudo cargar el reporte de etiquetas.
      </BaseAlert>

      <div v-if="tagRows.length > 0" class="tag-report-table">
        <div class="tag-report-table__header">
          <span>Etiqueta</span>
          <span>Ingresos</span>
          <span>Gastos</span>
          <span>Neto</span>
          <span>Movimientos</span>
        </div>

        <div
          v-for="row in tagRows"
          :key="row.tag.id"
          class="tag-report-table__row"
        >
          <strong class="report-tag" :style="{ '--tag-color': row.tag.color }">
            {{ row.tag.name }}
          </strong>
          <span>{{ formatMoney(row.incomeTotal, report?.currency ?? "CLP") }}</span>
          <span>{{ formatMoney(row.expenseTotal, report?.currency ?? "CLP") }}</span>
          <span>{{ formatMoney(row.netTotal, report?.currency ?? "CLP") }}</span>
          <span>{{ row.transactionCount }}</span>
        </div>
      </div>

      <p v-else class="reports-empty">Sin etiquetas para el periodo.</p>
    </section>
  </BaseCard>
</template>

<style scoped>
.reports-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--lu-space-3);
}

.reports-heading h2 {
  margin: 0;
  font-size: var(--lu-font-size-md);
}

.reports-heading span,
.reports-empty {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-sm);
}

.reports-alert {
  margin-block-end: var(--lu-space-4);
}

.tag-report-table {
  display: grid;
  overflow-x: auto;
}

.tag-report-table__header,
.tag-report-table__row {
  display: grid;
  grid-template-columns: minmax(10rem, 1.3fr) repeat(4, minmax(7rem, 1fr));
  gap: var(--lu-space-3);
  align-items: center;
  min-inline-size: 48rem;
  border-block-end: 1px solid var(--lu-color-border);
  padding: var(--lu-space-3);
}

.tag-report-table__header {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

.tag-report-table__row span {
  font-variant-numeric: tabular-nums;
}

.report-tag {
  --tag-color: #64748b;
  display: inline-flex;
  align-items: center;
  justify-self: start;
  min-block-size: 1.65rem;
  border: 1px solid color-mix(in srgb, var(--tag-color) 42%, transparent);
  border-radius: var(--lu-radius-pill);
  background: color-mix(in srgb, var(--tag-color) 14%, transparent);
  padding-inline: var(--lu-space-3);
  font-size: var(--lu-font-size-xs);
}
</style>
