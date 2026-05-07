<script setup lang="ts">
import { useQuery } from "@pinia/colada";
import { computed, ref } from "vue";

import { getMovements } from "@/api/ledger";
import BaseAlert from "@/components/base/BaseAlert.vue";
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import BaseCheckbox from "@/components/base/BaseCheckbox.vue";
import LedgerMovementsTable from "@/components/ledger/LedgerMovementsTable.vue";

const showOriginalCurrency = ref(false);
const baseCurrency = "CLP";

const movementsQuery = useQuery({
  key: ["movements"],
  query: getMovements,
});

const movements = computed(() => movementsQuery.data.value ?? []);
const movementCount = computed(() => movements.value.length);
</script>

<template>
  <BaseCard class="movements-panel">
    <template #header>
      <div class="movements-heading">
        <div>
          <h2 id="movements-heading">Movimientos</h2>
          <span>Todos los movimientos del libro mayor</span>
        </div>

        <div class="movements-actions">
          <BaseBadge>{{ movementCount }}</BaseBadge>
          <BaseCheckbox
            v-model:checked="showOriginalCurrency"
            label="Moneda original"
          />
        </div>
      </div>
    </template>

    <section aria-labelledby="movements-heading">
      <p class="currency-note">
        Valores mostrados en
        <strong>{{
          showOriginalCurrency ? "moneda original" : baseCurrency
        }}</strong>
      </p>

      <BaseAlert
        v-if="movementsQuery.error.value"
        class="movements-alert"
        tone="danger"
      >
        No se pudieron cargar los movimientos.
      </BaseAlert>

      <LedgerMovementsTable
        :movements="movements"
        :show-original-currency="showOriginalCurrency"
      />
    </section>
  </BaseCard>
</template>

<style scoped>
.movements-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-block-end: 0;
}

.movements-heading h2 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1rem;
}

.movements-heading span,
.currency-note {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.movements-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-3);
}

.currency-note {
  margin-block-end: var(--space-4);
}

.currency-note strong {
  color: var(--color-primary-strong);
}

.movements-alert {
  margin-block-end: var(--space-4);
}

@media (max-width: 680px) {
  .movements-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .movements-actions {
    justify-content: space-between;
  }
}
</style>
