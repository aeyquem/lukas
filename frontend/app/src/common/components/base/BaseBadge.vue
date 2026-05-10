<script setup lang="ts">
import { computed } from "vue";
import Badge from "primevue/badge";

const props = withDefaults(
  defineProps<{
    tone?: "neutral" | "success" | "warning" | "danger";
  }>(),
  {
    tone: "neutral",
  },
);

const severity = computed(() => {
  if (props.tone === "neutral") {
    return "secondary";
  }

  if (props.tone === "warning") {
    return "warn";
  }

  return props.tone;
});
</script>

<template>
  <Badge :severity="severity">
    <slot />
  </Badge>
</template>
