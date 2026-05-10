<script setup lang="ts">
import Select from "primevue/select";

defineOptions({
  inheritAttrs: false,
});

export type BaseSelectOption = {
  label: string;
  value: string;
};

const props = withDefaults(
  defineProps<{
    describedBy?: string;
    id?: string;
    invalid?: boolean;
    inputId?: string;
    modelValue: string;
    options: BaseSelectOption[];
    placeholder?: string;
  }>(),
  {
    describedBy: "",
    id: "",
    invalid: false,
    inputId: "",
    placeholder: "",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

function updateValue(value: string) {
  emit("update:modelValue", value);
}
</script>

<template>
  <Select
    v-bind="$attrs"
    :aria-describedby="describedBy"
    :aria-invalid="invalid"
    fluid
    :input-id="props.inputId || props.id || undefined"
    :invalid="invalid"
    :model-value="modelValue"
    option-label="label"
    option-value="value"
    :options="options"
    :placeholder="placeholder || undefined"
    @update:model-value="updateValue"
  />
</template>
