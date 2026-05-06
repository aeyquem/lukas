<script setup lang="ts">
withDefaults(
  defineProps<{
    tone?: "neutral" | "success" | "warning" | "danger";
  }>(),
  {
    tone: "neutral",
  },
);
</script>

<template>
  <span class="base-badge" :class="`base-badge--${tone}`">
    <slot />
  </span>
</template>

<style scoped>
.base-badge {
  display: inline-flex;
  align-items: center;
  min-block-size: 1.5rem;
  border: 1px solid transparent;
  border-radius: var(--radius-round);
  padding: 0 var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 800;
}

.base-badge--neutral {
  border-color: var(--color-border);
  background: var(--color-secondary);
  color: var(--color-secondary-foreground);
}

.base-badge--success {
  background: color-mix(in srgb, var(--color-success) 14%, transparent);
  color: var(--color-success);
}

.base-badge--warning {
  background: color-mix(in srgb, var(--color-warning) 16%, transparent);
  color: var(--color-warning);
}

.base-badge--danger {
  background: color-mix(in srgb, var(--color-danger) 14%, transparent);
  color: var(--color-danger);
}
</style>
