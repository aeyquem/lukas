<script setup lang="ts">
import Card from "primevue/card";
</script>

<template>
  <Card>
    <template v-if="$slots.header" #header>
      <slot name="header" />
    </template>
    <template #content>
      <slot />
    </template>
  </Card>
</template>
