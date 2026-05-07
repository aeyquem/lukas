<script setup lang="ts">
import BaseBadge from "../base/BaseBadge.vue";
import BaseField from "../base/BaseField.vue";
import BaseTextInput from "../base/BaseTextInput.vue";

const props = defineProps<{
  period: string;
  sectionTitle: string;
  sectionEyebrow: string;
}>();

const emit = defineEmits<{
  "update:period": [value: string];
}>();

function updatePeriod(value: string) {
  emit("update:period", value);
}
</script>

<template>
  <header class="app-header">
    <div class="header-title">
      <p class="eyebrow">{{ props.sectionEyebrow }}</p>
      <h1>{{ props.sectionTitle }}</h1>
    </div>

    <div class="header-context">
      <BaseBadge tone="success">CLP</BaseBadge>
      <BaseField input-id="period" label="Periodo">
        <BaseTextInput
          id="period"
          :model-value="props.period"
          type="month"
          @update:model-value="updatePeriod"
        />
      </BaseField>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  position: sticky;
  z-index: 8;
  inset-block-start: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  min-block-size: 4.75rem;
  border-block-end: 1px solid var(--color-border);
  background: color-mix(in srgb, var(--color-background) 96%, var(--color-card));
  padding: var(--space-3) var(--space-6);
}

.header-title {
  min-inline-size: 0;
}

.header-title h1 {
  margin: 0;
  color: var(--color-primary-strong);
  font-family: var(--font-display);
  font-size: clamp(1.75rem, 3vw, 2.35rem);
  font-weight: 400;
  line-height: 1;
}

.header-context {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.header-context :deep(.base-field) {
  min-inline-size: 11rem;
}

.eyebrow {
  margin: 0 0 var(--space-1);
  color: var(--color-muted-foreground);
  font-size: var(--font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

@media (max-width: 900px) {
  .app-header {
    position: static;
    align-items: stretch;
    flex-direction: column;
  }

  .header-context {
    align-items: stretch;
    flex-wrap: wrap;
  }

  .header-context :deep(.base-field) {
    flex: 1 1 12rem;
  }
}

@media (max-width: 680px) {
  .app-header {
    padding: var(--space-4);
  }
}
</style>
