<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string
    id: string
    label: string
    error?: string
  }>(),
  { error: undefined },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <fieldset class="opening-choice field" :aria-invalid="Boolean(error)" :aria-describedby="error ? `${id}-error` : undefined">
    <legend>{{ label }}</legend>
    <p class="field-hint">La fecha coincide o precede el saldo inicial. No inferimos si el movimiento ya estaba incluido.</p>
    <label :for="`${id}-apply`">
      <input
        :id="`${id}-apply`"
        type="radio"
        :name="id"
        value="apply_effect"
        :checked="modelValue === 'apply_effect'"
        :aria-invalid="Boolean(error)"
        :aria-describedby="error ? `${id}-error` : undefined"
        @change="emit('update:modelValue', 'apply_effect')"
      />
      Aplicar el movimiento
    </label>
    <label :for="`${id}-preserve`">
      <input
        :id="`${id}-preserve`"
        type="radio"
        :name="id"
        value="preserve_balance"
        :checked="modelValue === 'preserve_balance'"
        :aria-invalid="Boolean(error)"
        :aria-describedby="error ? `${id}-error` : undefined"
        @change="emit('update:modelValue', 'preserve_balance')"
      />
      Conservar el saldo inicial y compensar el monto
    </label>
    <p v-if="error" :id="`${id}-error`" class="field-error" role="alert">{{ error }}</p>
  </fieldset>
</template>

<style scoped>
.field { display: grid; gap: 7px; margin: 0 0 16px; }
.opening-choice { border: 1px solid #c9cec8; border-radius: 10px; padding: 12px 14px; }
.opening-choice legend { font-size: 14px; font-weight: 650; }
.opening-choice label { display: flex; align-items: flex-start; gap: 8px; font-weight: 450; line-height: 1.4; }
.opening-choice input { width: auto; min-height: auto; margin-top: 3px; }
.field-error { margin: 0; color: #a4372d; font-size: 13px; }
.field-hint { margin: 0 0 2px; color: #626b63; font-size: 13px; line-height: 1.45; }
</style>
