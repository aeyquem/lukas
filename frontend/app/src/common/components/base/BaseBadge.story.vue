<script setup lang="ts">
import BaseBadge from "@/common/components/base/BaseBadge.vue";
</script>

<template>
  <Story title="Base/Badge">
    <div class="story-grid">
      <BaseBadge>4 sobres</BaseBadge>
      <BaseBadge tone="success">CLP</BaseBadge>
      <BaseBadge tone="warning">Atención</BaseBadge>
      <BaseBadge tone="danger">Error</BaseBadge>
    </div>
  </Story>
</template>

<style scoped>
.story-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--lu-space-3);
}
</style>
