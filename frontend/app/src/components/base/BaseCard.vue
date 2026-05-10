<script setup lang="ts">
import Card from "primevue/card";
</script>

<template>
  <Card class="base-card">
    <template v-if="$slots.header" #header>
      <slot name="header" />
    </template>
    <template #content>
      <slot />
    </template>
  </Card>
</template>

<style scoped>
.base-card {
  --lu-card-padding: var(--lu-space-5);

  min-inline-size: 0;
}

.base-card :deep(.p-card-header) {
  padding: var(--lu-card-padding) var(--lu-card-padding) 0;
}

.base-card :deep(.p-card-header + .p-card-body) {
  padding-block-start: var(--lu-space-4);
}

.base-card :deep(.p-card-body) {
  min-inline-size: 0;
  padding: var(--lu-card-padding);
}

.base-card :deep(.p-card-content) {
  min-inline-size: 0;
  padding: 0;
}

@media (max-width: 680px) {
  .base-card {
    --lu-card-padding: var(--lu-space-4);
  }
}
</style>
