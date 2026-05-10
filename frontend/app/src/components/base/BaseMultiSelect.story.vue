<script setup lang="ts">
import { ref } from "vue";

import BaseMultiSelect, {
  type BaseMultiSelectOption,
} from "@/components/base/BaseMultiSelect.vue";

const selectedAccounts = ref(["checking", "cash"]);
const accountOptions: BaseMultiSelectOption[] = [
  { label: "Cuenta corriente", value: "checking" },
  { label: "Efectivo", value: "cash" },
  { label: "Tarjeta", value: "credit-card" },
];
</script>

<template>
  <Story title="Base/Multi Select">
    <div class="story-stack">
      <BaseMultiSelect
        v-model="selectedAccounts"
        display="chip"
        filter
        input-id="accounts-story"
        :max-selected-labels="2"
        :options="accountOptions"
        placeholder="Seleccionar"
        selected-items-label="Todas las cuentas"
      />
      <BaseMultiSelect
        :model-value="['checking']"
        disabled
        input-id="disabled-accounts-story"
        :options="accountOptions"
      />
    </div>
  </Story>
</template>

<style scoped>
.story-stack {
  display: grid;
  gap: 0.75rem;
  max-inline-size: 26rem;
}
</style>
