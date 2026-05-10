<script setup lang="ts">
import Checkbox from "primevue/checkbox";

defineOptions({
  inheritAttrs: false,
});

const props = withDefaults(
  defineProps<{
    checked: boolean;
    disabled?: boolean;
    inputId?: string;
    label: string;
  }>(),
  {
    disabled: false,
    inputId: "",
  },
);

const emit = defineEmits<{
  "update:checked": [value: boolean];
}>();

function updateChecked(value: boolean) {
  emit("update:checked", value);
}
</script>

<template>
  <label class="base-checkbox" :for="props.inputId || undefined">
    <Checkbox
      v-bind="$attrs"
      binary
      :disabled="disabled"
      :input-id="props.inputId || undefined"
      :model-value="checked"
      @update:model-value="updateChecked"
    />
    <strong>{{ label }}</strong>
  </label>
</template>

<style scoped>
.base-checkbox {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-sm);
}

.base-checkbox strong {
  font-weight: 700;
}
</style>
