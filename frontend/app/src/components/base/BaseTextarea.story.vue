<script setup lang="ts">
import { ref } from "vue";

import BaseTextarea from "@/components/base/BaseTextarea.vue";

const notes = ref("Notas internas del movimiento.");
</script>

<template>
  <Story title="Base/Textarea">
    <div class="story-stack">
      <BaseTextarea v-model="notes" aria-label="Notas" maxlength="1000" />
      <BaseTextarea
        model-value=""
        aria-label="Notas inválidas"
        described-by="textarea-error"
        invalid
      />
      <span id="textarea-error">La nota no puede superar el límite.</span>
      <BaseTextarea model-value="Solo lectura" aria-label="Notas disabled" disabled />
    </div>
  </Story>
</template>

<style scoped>
.story-stack {
  display: grid;
  gap: var(--lu-space-3);
  max-inline-size: 28rem;
}
</style>
