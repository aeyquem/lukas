<script setup lang="ts">
import BaseAlert from "@/common/components/base/BaseAlert.vue";
</script>

<template>
  <Story title="Base/Alert">
    <div class="story-stack">
      <BaseAlert>Información disponible.</BaseAlert>
      <BaseAlert tone="success">Cambios guardados.</BaseAlert>
      <BaseAlert tone="warning">Revisa los montos antes de continuar.</BaseAlert>
      <BaseAlert tone="danger">No se pudieron cargar los movimientos.</BaseAlert>
    </div>
  </Story>
</template>

<style scoped>
.story-stack {
  display: grid;
  gap: var(--lu-space-3);
}
</style>
