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
  emit("update:modelValue", (event.target as HTMLTextAreaElement).value);
}
</script>

<template>
  <textarea
    class="base-textarea"
    :aria-describedby="describedBy"
    :aria-invalid="invalid"
    :value="modelValue"
    @input="updateValue"
  ></textarea>
</template>

<style scoped>
.base-textarea {
  inline-size: 100%;
  min-block-size: 5.75rem;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-md);
  background: var(--color-input-background);
  color: var(--color-foreground);
  padding: var(--space-3);
  resize: vertical;
  transition:
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    box-shadow var(--motion-fast);
}

.base-textarea:focus-visible {
  border-color: var(--color-primary);
  outline: 3px solid color-mix(in srgb, var(--color-ring) 28%, transparent);
  outline-offset: 2px;
}

.base-textarea[aria-invalid="true"] {
  border-color: var(--color-danger);
}
</style>
