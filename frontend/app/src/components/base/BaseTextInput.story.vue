<script setup lang="ts">
import { ref } from "vue";

import BaseTextInput from "@/components/base/BaseTextInput.vue";

const name = ref("Cuenta corriente");
const amount = ref("125000");
const date = ref("2026-05-09");
const month = ref("2026-05");
</script>

<template>
  <Story title="Base/Text Input">
    <div class="story-stack">
      <BaseTextInput v-model="name" aria-label="Nombre" />
      <BaseTextInput
        v-model="amount"
        aria-label="Monto"
        inputmode="decimal"
        pattern="\d+(\.\d+)?"
        placeholder="0"
      />
      <BaseTextInput v-model="date" aria-label="Fecha" type="date" />
      <BaseTextInput v-model="month" aria-label="Periodo" type="month" />
      <BaseTextInput
        model-value="CLP"
        aria-label="Moneda deshabilitada"
        disabled
      />
      <BaseTextInput
        model-value=""
        aria-label="Nombre inválido"
        described-by="input-error"
        invalid
      />
      <span id="input-error">El nombre es obligatorio.</span>
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
