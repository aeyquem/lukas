<script setup lang="ts">
import { ref } from "vue";

import { sampleLedgerMovements } from "../../ledger/movements";
import LedgerMovementsTable from "./LedgerMovementsTable.vue";

const showOriginalCurrency = ref(false);
</script>

<template>
  <Story title="Ledger/Movements Table">
    <Variant title="Base currency">
      <LedgerMovementsTable :movements="sampleLedgerMovements" />
    </Variant>

    <Variant title="Original currency">
      <LedgerMovementsTable
        :movements="sampleLedgerMovements"
        show-original-currency
      />
    </Variant>

    <Variant title="Interactive controls">
      <div class="story-controls">
        <label>
          <input v-model="showOriginalCurrency" type="checkbox" />
          Moneda original
        </label>
      </div>
      <LedgerMovementsTable
        :movements="sampleLedgerMovements"
        :show-original-currency="showOriginalCurrency"
      />
    </Variant>
  </Story>
</template>

<style scoped>
.story-controls {
  display: flex;
  justify-content: flex-end;
  margin-block-end: var(--space-3);
}

.story-controls label {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-foreground);
  font-size: var(--font-size-sm);
  font-weight: 700;
}
</style>
