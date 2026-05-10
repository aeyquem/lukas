<script setup lang="ts">
import { ref } from "vue";

import BaseCheckbox from "@/components/base/BaseCheckbox.vue";

const checked = ref(true);
</script>

<template>
  <Story title="Base/Checkbox">
    <div class="story-stack">
      <BaseCheckbox
        v-model:checked="checked"
        input-id="show-original-currency-story"
        label="Mostrar moneda original"
      />
      <BaseCheckbox
        checked
        disabled
        input-id="disabled-checkbox-story"
        label="Opción deshabilitada"
      />
    </div>
  </Story>
</template>

<style scoped>
.story-stack {
  display: grid;
  gap: var(--space-3);
}
</style>
