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
  gap: var(--space-2);
}

.base-field__label {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.base-field__hint,
.base-field__error {
  font-size: var(--font-size-xs);
  line-height: 1.4;
}

.base-field__hint {
  color: var(--color-muted-foreground);
}

.base-field__error {
  color: var(--color-danger);
  font-weight: 700;
}
</style>
