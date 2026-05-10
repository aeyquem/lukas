<script setup lang="ts">
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseCard from "@/components/base/BaseCard.vue";
</script>

<template>
  <Story title="Base/Card">
    <BaseCard>
      <template #header>
        <div class="story-heading">
          <h2>Cuentas</h2>
          <BaseBadge>3</BaseBadge>
        </div>
      </template>
      <p>Todas las cuentas registradas.</p>
    </BaseCard>
  </Story>
</template>

<style scoped>
.story-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 1rem;
}

.story-heading h2,
p {
  margin: 0;
}
</style>
