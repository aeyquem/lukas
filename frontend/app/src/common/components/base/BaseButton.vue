<script setup lang="ts">
import { computed } from "vue";
import Button from "primevue/button";

const props = withDefaults(
  defineProps<{
    disabled?: boolean;
    loading?: boolean;
    size?: "sm" | "md";
    type?: "button" | "submit" | "reset";
    variant?: "primary" | "secondary" | "ghost" | "danger";
  }>(),
  {
    disabled: false,
    loading: false,
    size: "md",
    type: "button",
    variant: "primary",
  },
);

const severity = computed(() => {
  if (props.variant === "danger") {
    return "danger";
  }

  if (props.variant === "secondary" || props.variant === "ghost") {
    return "secondary";
  }

  return undefined;
});

const text = computed(() => props.variant === "ghost");
const outlined = computed(() => props.variant === "secondary");
const primeSize = computed(() => (props.size === "sm" ? "small" : undefined));
</script>

<template>
  <Button
    :aria-busy="loading"
    :disabled="disabled || loading"
    :loading="loading"
    :outlined="outlined"
    :severity="severity"
    :size="primeSize"
    :text="text"
    :type="type"
  >
    <slot />
  </Button>
</template>
