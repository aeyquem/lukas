<script setup lang="ts">
import { ref } from "vue";

import BaseSelect, {
  type BaseSelectOption,
} from "@/common/components/base/BaseSelect.vue";

const accountType = ref("bank");
const category = ref("");
const accountTypeOptions: BaseSelectOption[] = [
  { label: "Banco", value: "bank" },
  { label: "Efectivo", value: "cash" },
  { label: "Tarjeta", value: "credit_card" },
];
const categoryOptions: BaseSelectOption[] = [
  { label: "Sin categoría", value: "" },
  { label: "Arriendo", value: "env-rent" },
  { label: "Mercado", value: "env-market" },
];
</script>

<template>
  <Story title="Base/Select">
    <div class="story-stack">
      <BaseSelect
        id="account-type-story"
        v-model="accountType"
        :options="accountTypeOptions"
      />
      <BaseSelect
        id="category-story"
        v-model="category"
        :options="categoryOptions"
        placeholder="Seleccionar categoría"
      />
      <BaseSelect
        id="invalid-select-story"
        model-value=""
        described-by="select-error"
        invalid
        :options="categoryOptions"
      />
      <span id="select-error">Selecciona una opción válida.</span>
      <BaseSelect
        id="disabled-select-story"
        model-value="bank"
        disabled
        :options="accountTypeOptions"
      />
    </div>
  </Story>
</template>

<style scoped>
.story-stack {
  display: grid;
  gap: var(--lu-space-3);
  max-inline-size: 24rem;
}
</style>
