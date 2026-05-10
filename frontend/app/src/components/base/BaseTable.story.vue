<script setup lang="ts">
import { ref } from "vue";

import BaseColumn from "@/components/base/BaseColumn.vue";
import BaseDataTable from "@/components/base/BaseDataTable.vue";
import BaseTreeTable from "@/components/base/BaseTreeTable.vue";

const rows = [
  { id: "mov-1", account: "Cuenta corriente", amount: "$120.000" },
  { id: "mov-2", account: "Efectivo", amount: "$24.000" },
];
const treeRows = ref([
  {
    key: "cat-living",
    data: { name: "Vida diaria", assigned: "$300.000" },
    children: [
      {
        key: "env-market",
        data: { name: "Mercado", assigned: "$180.000" },
      },
    ],
  },
]);
</script>

<template>
  <Story title="Base/Table">
    <div class="story-stack">
      <BaseDataTable data-key="id" :value="rows">
        <BaseColumn field="account" header="Cuenta" />
        <BaseColumn field="amount" header="Monto" />
      </BaseDataTable>

      <BaseTreeTable :value="treeRows">
        <BaseColumn field="name" header="Sobre" expander />
        <BaseColumn field="assigned" header="Asignado" />
      </BaseTreeTable>
    </div>
  </Story>
</template>

<style scoped>
.story-stack {
  display: grid;
  gap: var(--space-4);
}
</style>
