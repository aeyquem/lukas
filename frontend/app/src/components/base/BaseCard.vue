<template>
  <article class="base-card">
    <header v-if="$slots.header" class="base-card__header">
      <slot name="header" />
    </header>
    <div class="base-card__body">
      <slot />
    </div>
  </article>
</template>

<style scoped>
.base-card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-card);
  box-shadow: var(--shadow-sm);
}

.base-card__header {
  border-block-end: 1px solid var(--color-border);
  padding: var(--space-4);
}

.base-card__body {
  padding: var(--space-4);
}
</style>
