<script setup lang="ts">
import { ref } from "vue";

import LedgerMovementsTable from "@/features/ledger/components/LedgerMovementsTable.vue";
import type { LedgerMovement } from "@/features/ledger/model/movements";

const showOriginalCurrency = ref(false);

const sampleLedgerMovements: LedgerMovement[] = [
  {
    id: "movement-001",
    transactionId: "transaction-001",
    date: "2026-05-02",
    accountId: "account-001",
    accountName: "Cuenta Corriente",
    kind: "income",
    accountEffect: "increase",
    categoryName: "Ingresos",
    description: "Sueldo mensual",
    direction: "in",
    originalAmount: "1850000",
    originalCurrency: "CLP",
    baseAmount: "1850000",
    baseCurrency: "CLP",
    tags: [
      {
        id: "tag-income",
        name: "Trabajo",
        normalizedName: "trabajo",
        color: "#2563EB",
      },
    ],
  },
  {
    id: "movement-002",
    transactionId: "transaction-002",
    date: "2026-05-03",
    accountId: "account-002",
    accountName: "Tarjeta Visa",
    kind: "expense",
    accountEffect: "decrease",
    description: "Suscripcion software",
    direction: "out",
    originalAmount: "24.99",
    originalCurrency: "USD",
    baseAmount: "24.99",
    baseCurrency: "USD",
    tags: [],
  },
  {
    id: "movement-003",
    transactionId: "transaction-003",
    date: "2026-05-04",
    accountId: "account-003",
    accountName: "Efectivo",
    kind: "expense",
    accountEffect: "decrease",
    categoryName: "Comida",
    description: "Almuerzo",
    direction: "out",
    originalAmount: "8500",
    originalCurrency: "CLP",
    baseAmount: "8500",
    baseCurrency: "CLP",
    tags: [
      {
        id: "tag-food",
        name: "Comida",
        normalizedName: "comida",
        color: "#16A34A",
      },
    ],
  },
  {
    id: "movement-004",
    transactionId: "transaction-004",
    date: "2026-05-05",
    accountId: "account-004",
    accountName: "Cuenta USD",
    kind: "income",
    accountEffect: "increase",
    categoryName: "Trabajo",
    description: "Reembolso proveedor",
    direction: "in",
    originalAmount: "110.5",
    originalCurrency: "USD",
    baseAmount: "110.5",
    baseCurrency: "USD",
    tags: [
      {
        id: "tag-work",
        name: "Trabajo",
        normalizedName: "trabajo",
        color: "#2563EB",
      },
    ],
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
  margin-block-end: var(--lu-space-3);
}

.story-controls label {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-sm);
  font-weight: 700;
}
</style>
