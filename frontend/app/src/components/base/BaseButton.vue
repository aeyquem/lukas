<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    disabled?: boolean;
    loading?: boolean;
    size?: "sm" | "md";
    type?: "button" | "submit" | "reset";
    variant?: "primary" | "secondary" | "ghost" | "danger";
  }>(),
  {
    disabled: false,
    loading: false,
    size: "md",
    type: "button",
    variant: "primary",
  },
);

const buttonClass = computed(() => [
  "base-button",
  `base-button--${props.variant}`,
  `base-button--${props.size}`,
]);
</script>

<template>
  <button
    :aria-busy="loading"
    :class="buttonClass"
    :disabled="disabled || loading"
    :type="type"
  >
    <span v-if="loading" class="base-button__spinner" aria-hidden="true"></span>
    <slot />
  </button>
</template>

<style scoped>
.base-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  min-block-size: 2.75rem;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 700;
  line-height: 1;
  text-decoration: none;
  transition:
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    box-shadow var(--motion-fast),
    color var(--motion-fast),
    opacity var(--motion-fast);
}

.base-button--sm {
  min-block-size: 2.25rem;
  padding: 0 var(--space-3);
  font-size: var(--font-size-sm);
}

.base-button--md {
  padding: 0 var(--space-4);
  font-size: var(--font-size-md);
}

.base-button--primary {
  background: var(--color-primary);
  color: var(--color-primary-foreground);
  box-shadow: var(--shadow-sm);
}

.base-button--primary:hover,
.base-button--primary:focus-visible {
  background: var(--color-primary-strong);
  box-shadow: var(--shadow-focus);
}

.base-button--secondary {
  border-color: var(--color-border);
  background: var(--color-secondary);
  color: var(--color-secondary-foreground);
}

.base-button--secondary:hover,
.base-button--secondary:focus-visible {
  border-color: color-mix(in srgb, var(--color-primary) 34%, transparent);
  background: var(--color-muted);
}

.base-button--ghost {
  background: transparent;
  color: var(--color-muted-foreground);
}

.base-button--ghost:hover,
.base-button--ghost:focus-visible {
  background: var(--color-secondary);
  color: var(--color-foreground);
}

.base-button--danger {
  background: var(--color-danger);
  color: var(--color-danger-foreground);
}

.base-button--danger:hover,
.base-button--danger:focus-visible {
  background: var(--color-danger-strong);
}

.base-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.base-button__spinner {
  inline-size: 1rem;
  block-size: 1rem;
  border: 2px solid currentColor;
  border-block-start-color: transparent;
  border-radius: var(--radius-round);
  animation: button-spin 700ms linear infinite;
}

@keyframes button-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .base-button__spinner {
    animation-duration: 1ms;
  }
}
</style>
