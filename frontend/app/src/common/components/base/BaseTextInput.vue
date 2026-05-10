<script setup lang="ts">
import InputText from "primevue/inputtext";

defineOptions({
  inheritAttrs: false,
});

withDefaults(
  defineProps<{
    describedBy?: string;
    invalid?: boolean;
    modelValue: string;
    type?: string;
  }>(),
  {
    describedBy: "",
    invalid: false,
    type: "text",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

function updateValue(value: string | undefined) {
  emit("update:modelValue", value ?? "");
}
</script>

<template>
  <InputText
    v-bind="$attrs"
    :aria-describedby="describedBy"
    :aria-invalid="invalid"
    fluid
    :invalid="invalid"
    :model-value="modelValue"
    :type="type"
    @update:model-value="updateValue"
  />
</template>
