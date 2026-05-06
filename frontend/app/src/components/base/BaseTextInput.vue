<script setup lang="ts">
withDefaults(
  defineProps<{
    describedBy?: string;
    invalid?: boolean;
    modelValue: string;
    type?: string;
  }>(),
  {
    describedBy: "",
    invalid: false,
    type: "text",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

function updateValue(event: Event) {
  emit("update:modelValue", (event.target as HTMLInputElement).value);
}
</script>

<template>
  <input
    class="base-control"
    :aria-describedby="describedBy"
    :aria-invalid="invalid"
    :type="type"
    :value="modelValue"
    @input="updateValue"
  />
</template>

<style scoped>
.base-control {
  inline-size: 100%;
  min-block-size: 2.75rem;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-md);
  background: var(--color-input-background);
  color: var(--color-foreground);
  padding: 0 var(--space-3);
  transition:
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    box-shadow var(--motion-fast);
}

.base-control::placeholder {
  color: color-mix(in srgb, var(--color-muted-foreground) 68%, transparent);
}

.base-control:hover {
  border-color: color-mix(in srgb, var(--color-primary) 35%, var(--color-border));
}

.base-control:focus-visible {
  border-color: var(--color-primary);
  outline: 3px solid color-mix(in srgb, var(--color-ring) 28%, transparent);
  outline-offset: 2px;
}

.base-control[aria-invalid="true"] {
  border-color: var(--color-danger);
}
</style>
