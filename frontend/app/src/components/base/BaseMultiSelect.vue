<script setup lang="ts">
import MultiSelect from "primevue/multiselect";

defineOptions({
  inheritAttrs: false,
});

export type BaseMultiSelectOption = {
  label: string;
  value: string;
};

withDefaults(
  defineProps<{
    disabled?: boolean;
    display?: "comma" | "chip";
    filter?: boolean;
    inputId?: string;
    invalid?: boolean;
    maxSelectedLabels?: number;
    modelValue: string[];
    options: BaseMultiSelectOption[];
    placeholder?: string;
    selectedItemsLabel?: string;
  }>(),
  {
    disabled: false,
    display: "comma",
    filter: false,
    inputId: "",
    invalid: false,
    maxSelectedLabels: undefined,
    placeholder: "",
    selectedItemsLabel: "",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string[]];
}>();

function updateValue(value: string[]) {
  emit("update:modelValue", value);
}
</script>

<template>
  <MultiSelect
    v-bind="$attrs"
    :disabled="disabled"
    :display="display"
    :filter="filter"
    fluid
    :input-id="inputId || undefined"
    :invalid="invalid"
    :max-selected-labels="maxSelectedLabels"
    :model-value="modelValue"
    option-label="label"
    option-value="value"
    :options="options"
    :placeholder="placeholder || undefined"
    :selected-items-label="selectedItemsLabel || undefined"
    @update:model-value="updateValue"
  />
</template>
