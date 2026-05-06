<script setup lang="ts">
withDefaults(
  defineProps<{
    tone?: "info" | "success" | "warning" | "danger";
  }>(),
  {
    tone: "info",
  },
);
</script>

<template>
  <p class="base-alert" :class="`base-alert--${tone}`">
    <slot />
  </p>
</template>

<style scoped>
.base-alert {
  border: 1px solid var(--alert-border);
  border-radius: var(--radius-md);
  background: var(--alert-background);
  color: var(--alert-foreground);
  font-size: var(--font-size-sm);
  font-weight: 700;
  line-height: 1.45;
  margin: 0;
  padding: var(--space-3);
}

.base-alert--info {
  --alert-background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  --alert-border: color-mix(in srgb, var(--color-primary) 25%, transparent);
  --alert-foreground: var(--color-primary-strong);
}

.base-alert--success {
  --alert-background: color-mix(in srgb, var(--color-success) 10%, transparent);
  --alert-border: color-mix(in srgb, var(--color-success) 25%, transparent);
  --alert-foreground: var(--color-success);
}

.base-alert--warning {
  --alert-background: color-mix(in srgb, var(--color-warning) 10%, transparent);
  --alert-border: color-mix(in srgb, var(--color-warning) 25%, transparent);
  --alert-foreground: var(--color-warning);
}

.base-alert--danger {
  --alert-background: color-mix(in srgb, var(--color-danger) 10%, transparent);
  --alert-border: color-mix(in srgb, var(--color-danger) 25%, transparent);
  --alert-foreground: var(--color-danger);
}
</style>
