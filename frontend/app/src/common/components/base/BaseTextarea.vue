<script setup lang="ts">
import Textarea from "primevue/textarea";

defineOptions({
  inheritAttrs: false,
});

withDefaults(
  defineProps<{
    describedBy?: string;
    invalid?: boolean;
    modelValue: string;
  }>(),
  {
    describedBy: "",
    invalid: false,
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
  <Textarea
    v-bind="$attrs"
    :aria-describedby="describedBy"
    :aria-invalid="invalid"
    fluid
    :invalid="invalid"
    :model-value="modelValue"
    @update:model-value="updateValue"
  />
</template>
