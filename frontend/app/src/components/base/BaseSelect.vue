<script setup lang="ts">
withDefaults(
  defineProps<{
    describedBy?: string;
    invalid?: boolean;
    modelValue: string;
  }>(),
  {
    describedBy: "",
    invalid: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

function updateValue(event: Event) {
  emit("update:modelValue", (event.target as HTMLSelectElement).value);
}
</script>

<template>
  <select
    class="base-select"
    :aria-describedby="describedBy"
    :aria-invalid="invalid"
    :value="modelValue"
    @change="updateValue"
  >
    <slot />
  </select>
</template>

<style scoped>
.base-select {
  inline-size: 100%;
  min-block-size: 2.75rem;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-md);
  appearance: none;
  background:
    linear-gradient(45deg, transparent 50%, var(--color-muted-foreground) 50%)
      calc(100% - 1.15rem) 1.16rem / 0.38rem 0.38rem no-repeat,
    linear-gradient(135deg, var(--color-muted-foreground) 50%, transparent 50%)
      calc(100% - 0.9rem) 1.16rem / 0.38rem 0.38rem no-repeat,
    var(--color-input-background);
  color: var(--color-foreground);
  padding: 0 2.5rem 0 var(--space-3);
  transition:
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    box-shadow var(--motion-fast);
}

.base-select:focus-visible {
  border-color: var(--color-primary);
  outline: 3px solid color-mix(in srgb, var(--color-ring) 28%, transparent);
  outline-offset: 2px;
}

.base-select[aria-invalid="true"] {
  border-color: var(--color-danger);
}
</style>
