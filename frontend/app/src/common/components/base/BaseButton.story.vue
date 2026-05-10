<script setup lang="ts">
import BaseButton from "@/common/components/base/BaseButton.vue";
</script>

<template>
  <Story title="Base/Button">
    <div class="story-grid">
      <BaseButton>Guardar</BaseButton>
      <BaseButton variant="secondary">Cancelar</BaseButton>
      <BaseButton variant="ghost">Ver detalle</BaseButton>
      <BaseButton variant="danger">Archivar</BaseButton>
      <BaseButton size="sm">Agregar cuenta</BaseButton>
      <BaseButton loading>Guardando</BaseButton>
      <BaseButton disabled>Deshabilitado</BaseButton>
      <BaseButton type="submit">Enviar formulario</BaseButton>
    </div>
  </Story>
</template>

<style scoped>
.story-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--lu-space-3);
  align-items: center;
}
</style>
