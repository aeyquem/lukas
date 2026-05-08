<script setup lang="ts">
import { ref } from "vue";

import LedgerMovementsTable from "@/components/ledger/LedgerMovementsTable.vue";
import type { LedgerMovement } from "@/ledger/movements";

const showOriginalCurrency = ref(false);

const sampleLedgerMovements: LedgerMovement[] = [
  {
    id: "movement-001",
    transactionId: "transaction-001",
    date: "2026-05-02",
    accountId: "account-001",
    accountName: "Cuenta Corriente",
    categoryName: "Ingresos",
    description: "Sueldo mensual",
    direction: "in",
    originalAmount: "1850000",
    originalCurrency: "CLP",
    baseAmount: "1850000",
    baseCurrency: "CLP",
  },
  {
    id: "movement-002",
    transactionId: "transaction-002",
    date: "2026-05-03",
    accountId: "account-002",
    accountName: "Tarjeta Visa",
    description: "Suscripcion software",
    direction: "out",
    originalAmount: "24.99",
    originalCurrency: "USD",
    baseAmount: "24.99",
    baseCurrency: "USD",
  },
  {
    id: "movement-003",
    transactionId: "transaction-003",
    date: "2026-05-04",
    accountId: "account-003",
    accountName: "Efectivo",
    categoryName: "Comida",
    description: "Almuerzo",
    direction: "out",
    originalAmount: "8500",
    originalCurrency: "CLP",
    baseAmount: "8500",
    baseCurrency: "CLP",
  },
  {
    id: "movement-004",
    transactionId: "transaction-004",
    date: "2026-05-05",
    accountId: "account-004",
    accountName: "Cuenta USD",
    categoryName: "Trabajo",
    description: "Reembolso proveedor",
    direction: "in",
    originalAmount: "110.5",
    originalCurrency: "USD",
    baseAmount: "110.5",
    baseCurrency: "USD",
  },
];
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
