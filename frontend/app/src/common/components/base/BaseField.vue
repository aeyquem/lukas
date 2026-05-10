<script setup lang="ts">
withDefaults(
  defineProps<{
    error?: string;
    hint?: string;
    inputId: string;
    label: string;
  }>(),
  {
    error: "",
    hint: "",
  },
);
</script>

<template>
  <label class="base-field" :for="inputId">
    <span class="base-field__label">{{ label }}</span>
    <slot
      :described-by="
        error ? `${inputId}-error` : hint ? `${inputId}-hint` : ''
      "
    />
    <span v-if="error" :id="`${inputId}-error`" class="base-field__error">
      {{ error }}
    </span>
    <span v-else-if="hint" :id="`${inputId}-hint`" class="base-field__hint">
      {{ hint }}
    </span>
  </label>
</template>

<style scoped>
.base-field {
  display: grid;
  gap: var(--lu-space-2);
}

.base-field__label {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-sm);
  font-weight: 700;
}

.base-field__hint,
.base-field__error {
  font-size: var(--lu-font-size-xs);
  line-height: var(--lu-line-normal);
}

.base-field__hint {
  color: var(--lu-color-text-muted);
}

.base-field__error {
  color: var(--lu-color-danger);
  font-weight: 700;
}
</style>
