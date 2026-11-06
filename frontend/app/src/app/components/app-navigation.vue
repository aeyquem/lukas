<script setup lang="ts">
defineProps<{ dashboardPeriod: string }>()
</script>

<template>
  <nav aria-label="Navegación principal">
    <RouterLink :to="{ name: 'home', query: { dashboardPeriod } }">Panel</RouterLink>
    <RouterLink :to="{ name: 'budget' }">Presupuesto</RouterLink>
    <RouterLink :to="{ name: 'accounts' }">Cuentas</RouterLink>
    <RouterLink :to="{ name: 'transactions' }">Transacciones</RouterLink>
    <RouterLink :to="{ name: 'payees' }">Payees</RouterLink>
    <RouterLink :to="{ name: 'projections' }">Proyecciones</RouterLink>
    <RouterLink :to="{ name: 'objectives' }">Objetivos</RouterLink>
  </nav>
</template>

<style scoped>
nav {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
}
nav a {
  border-radius: var(--control-radius);
  padding: 11px 13px;
  color: var(--muted);
  font-size: 13px;
  font-weight: 600;
  text-decoration: none;
}
nav a:hover {
  background: var(--surface-soft);
  color: var(--text);
}
nav a[aria-current='page'] {
  color: var(--on-accent);
  background: var(--accent);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 20%);
}
@media (max-width: 1050px) {
  nav { order: 4; width: 100%; justify-content: center; }
}
@media (max-width: 480px) {
  nav { gap: 2px; }
  nav a { padding: 10px 8px; font-size: 11px; }
}
</style>
