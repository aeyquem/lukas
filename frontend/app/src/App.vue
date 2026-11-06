<template>
  <div class="app-shell">
    <header class="app-header">
      <RouterLink class="brand" :to="{ name: 'home' }" aria-label="Lukas, inicio">
        <span class="brand-mark" aria-hidden="true">L</span>
        <span class="brand-name">Lukas</span>
      </RouterLink>

      <nav aria-label="Navegación principal">
        <RouterLink :to="{ name: 'accounts' }">Cuentas</RouterLink>
        <RouterLink :to="{ name: 'transactions' }">Movimientos</RouterLink>
        <RouterLink :to="{ name: 'budget' }">Presupuesto</RouterLink>
      </nav>
    </header>

    <RouterView />
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
}

.app-header {
  display: flex;
  width: min(100% - 40px, 1120px);
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  margin-inline: auto;
  border-bottom: 1px solid #dcddd7;
}

.brand {
  padding-block: 18px;
  text-decoration: none;
}

nav {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 20px;
}

nav a {
  padding-block: 10px;
  color: #52675b;
  font-size: 14px;
  font-weight: 600;
  text-decoration: none;
}

nav a[aria-current='page'] {
  color: #24573f;
  text-decoration: underline;
  text-underline-offset: 5px;
}

@media (max-width: 560px) {
  .app-header {
    align-items: flex-start;
    flex-direction: column;
    gap: 0;
  }

  nav {
    padding-bottom: 12px;
  }
}
</style>
