<script setup lang="ts">
withDefaults(
  defineProps<{
    checked: boolean;
    disabled?: boolean;
    label: string;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{
  "update:checked": [value: boolean];
}>();

function updateChecked(event: Event) {
  emit("update:checked", (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <label class="base-checkbox">
    <input
      :checked="checked"
      :disabled="disabled"
      type="checkbox"
      @change="updateChecked"
    />
    <span aria-hidden="true"></span>
    <strong>{{ label }}</strong>
  </label>
</template>

<style scoped>
.base-checkbox {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-foreground);
  cursor: pointer;
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.base-checkbox input {
  position: absolute;
  inline-size: 1px;
  block-size: 1px;
  clip: rect(0 0 0 0);
  clip-path: inset(50%);
  overflow: hidden;
}

.base-checkbox span {
  display: inline-grid;
  place-items: center;
  inline-size: 1.15rem;
  block-size: 1.15rem;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  background: var(--color-input-background);
  transition:
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    box-shadow var(--motion-fast);
}

.base-checkbox span::after {
  inline-size: 0.42rem;
  block-size: 0.7rem;
  border-block-end: 2px solid var(--color-primary-foreground);
  border-inline-end: 2px solid var(--color-primary-foreground);
  content: "";
  opacity: 0;
  transform: rotate(45deg) translate(-1px, -1px);
}

.base-checkbox input:checked + span {
  border-color: var(--color-primary);
  background: var(--color-primary);
}

.base-checkbox input:checked + span::after {
  opacity: 1;
}

.base-checkbox input:focus-visible + span {
  outline: 3px solid color-mix(in srgb, var(--color-ring) 28%, transparent);
  outline-offset: 2px;
}

.base-checkbox:has(input:disabled) {
  cursor: not-allowed;
  opacity: 0.55;
}
</style>
