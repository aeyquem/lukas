<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const title = computed(() => String(route.meta.title ?? 'Lukas'))
const description = computed(() => String(route.meta.description ?? ''))
</script>

<template>
  <main class="page-shell feature-page" aria-labelledby="feature-title">
    <p class="eyebrow">Lukas</p>
    <h1 id="feature-title">{{ title }}</h1>
    <p class="feature-copy">{{ description }}</p>
    <p class="feature-status" role="status">Esta sección se está preparando.</p>
  </main>
</template>

<style scoped>
.feature-page {
  padding-block: clamp(64px, 14vh, 140px);
}

h1 {
  margin: 0;
  font-size: clamp(38px, 6vw, 64px);
  font-weight: 560;
  letter-spacing: -0.06em;
}

.feature-copy {
  margin: 20px 0 0;
  color: #616760;
  font-size: 18px;
  line-height: 1.6;
}

.feature-status {
  margin-top: 36px;
  color: #52675b;
}
</style>
