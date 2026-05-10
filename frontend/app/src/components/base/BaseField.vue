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
  gap: 0.5rem;
}

.base-field__label {
  color: var(--p-text-muted-color);
  font-size: 0.875rem;
  font-weight: 700;
}

.base-field__hint,
.base-field__error {
  font-size: 0.75rem;
  line-height: 1.4;
}

.base-field__hint {
  color: var(--p-text-muted-color);
}

.base-field__error {
  color: var(--p-red-500);
  font-weight: 700;
}
</style>
