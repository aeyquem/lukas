<script setup lang="ts">
import { computed } from "vue";
import Message from "primevue/message";

const props = withDefaults(
  defineProps<{
    tone?: "info" | "success" | "warning" | "danger";
  }>(),
  {
    tone: "info",
  },
);

const severity = computed(() => {
  if (props.tone === "danger") {
    return "error";
  }

  if (props.tone === "warning") {
    return "warn";
  }

  return props.tone;
});
</script>

<template>
  <Message :closable="false" :severity="severity">
    <slot />
  </Message>
</template>
