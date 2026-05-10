<script setup lang="ts">
import {
  amountParts,
  assignedInputDisplaySize,
  assignedInputValue,
} from "@/features/budget/model/amounts";
import BaseTextInput from "@/common/components/base/BaseTextInput.vue";

defineProps<{
  amount: string;
  isEditing: boolean;
  label: string;
}>();

defineEmits<{
  blur: [];
  edit: [];
  updateAmount: [amount: string];
}>();

const vAssignedInputFocus = {
  mounted(element: HTMLElement) {
    element.focus();

    if (element instanceof HTMLInputElement) {
      element.select();
    }
  },
};
</script>

<template>
  <BaseTextInput
    v-if="isEditing"
    v-assigned-input-focus
    :aria-label="`Asignado para ${label}`"
    class="assigned-input"
    inputmode="numeric"
    :maxlength="assignedInputDisplaySize"
    :model-value="assignedInputValue(amount)"
    pattern="[0-9.]*"
    :size="assignedInputDisplaySize"
    type="text"
    @blur="$emit('blur')"
    @update:model-value="$emit('updateAmount', $event)"
  />
  <button
    v-else
    class="assigned-display numeric"
    type="button"
    :aria-label="`Editar asignado para ${label}`"
    @click="$emit('edit')"
  >
    <span class="numeric__prefix">
      {{ amountParts(amount).prefix }}
    </span>
    <span class="numeric__amount">
      {{ amountParts(amount).number }}
    </span>
  </button>
</template>

<style scoped>
.numeric {
  display: inline-flex;
  align-items: baseline;
  justify-content: flex-end;
  gap: var(--lu-space-1);
  color: var(--lu-color-text);
  inline-size: 100%;
  white-space: nowrap;
}

.numeric__amount {
  text-align: end;
}

.assigned-display {
  border: 0;
  background: transparent;
  cursor: text;
  padding: 0;
}

.assigned-display:hover,
.assigned-display:focus-visible {
  color: var(--lu-color-primary);
  outline: 0;
}

.assigned-display:focus-visible {
  text-decoration: underline;
  text-underline-offset: 0.2em;
}

.assigned-input {
  box-sizing: content-box;
  inline-size: 10ch;
  max-inline-size: 10rem;
  min-block-size: var(--lu-size-control-sm);
  min-inline-size: 10ch;
  text-align: end;
}
</style>
