<script setup lang="ts">
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseField from "@/components/base/BaseField.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";

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
  gap: 1rem;
  min-block-size: 4.75rem;
  border-block-end: 1px solid var(--p-content-border-color);
  background: color-mix(
    in srgb,
    var(--p-content-background) 92%,
    transparent
  );
  padding: 0.75rem 1.5rem;
}

.header-title {
  min-inline-size: 0;
}

.header-title h1 {
  margin: 0;
  font-size: clamp(1.75rem, 3vw, 2.35rem);
  font-weight: 700;
  line-height: 1;
}

.header-context {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.header-context :deep(.base-field) {
  min-inline-size: 11rem;
}

.eyebrow {
  margin: 0 0 0.25rem;
  color: var(--p-text-muted-color);
  font-size: 0.75rem;
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
    padding: 1rem;
  }
}
</style>
