<script setup lang="ts">
import AppNavigation from './app/components/app-navigation.vue'
import DashboardPeriodControl from './app/components/dashboard-period-control.vue'
import AppearanceMenu from './app/components/appearance-menu.vue'
import { useDashboardPeriod } from './app/use-dashboard-period'
const dashboardPeriod = useDashboardPeriod()
</script>

<template>
  <div class="app-shell">
    <a class="skip-link" href="#main-content">Ir al contenido</a>
    <header class="app-header island">
      <RouterLink class="brand" :to="{ name: 'home' }" aria-label="Lukas, inicio">
        <img class="brand-mark" :src="'/ignacio-carrera-pinto.svg'" alt="" />
        <span class="brand-name">lukas<span class="brand-dot">.</span></span>
      </RouterLink>

      <DashboardPeriodControl :dashboard-period="dashboardPeriod" />
      <AppNavigation :dashboard-period="dashboardPeriod" />
      <AppearanceMenu />
    </header>

    <div id="main-content" class="app-content" tabindex="-1"><RouterView /></div>

    <footer class="status-bar">
      <span><span class="status-mark" aria-hidden="true"></span>Espacio personal</span>
      <span class="status-context">Hecho para ir a tu ritmo.</span>
      <span>Santiago · CLP</span>
    </footer>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100dvh;
  max-width: 1680px;
  margin-inline: auto;
  padding: var(--shell-padding);
  display: flex;
  flex-direction: column;
  gap: var(--island-gap);
}
.app-header {
  display: flex;
  align-items: center;
  gap: clamp(16px, 2.5vw, 40px);
  padding: 14px 20px;
  position: relative;
  z-index: 2;
}
.brand {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  text-decoration: none;
  flex-shrink: 0;
}
.brand-mark {
  display: block;
  width: 48px;
  height: 48px;
  border-radius: 14%;
  object-fit: cover;
}
.brand-name {
  font-size: 28px;
  font-weight: 650;
  letter-spacing: -.065em;
}
.brand-dot {
  color: var(--accent);
}
.app-content {
  flex: 1;
  min-width: 0;
  outline: none;
}
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px;
  padding: 4px 12px;
  color: var(--muted);
  font-size: 11px;
}
.status-bar > span:first-child {
  display: flex;
  align-items: center;
  gap: 7px;
}
.status-mark {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
}
.skip-link {
  position: fixed;
  top: -100px;
  left: 24px;
  z-index: 10;
  padding: 12px;
  border-radius: var(--control-radius);
  background: var(--surface);
}
.skip-link:focus {
  top: 12px;
}
@media (max-width: 1050px) {
  .app-header {
    flex-wrap: wrap;
    gap: 12px;
  }
}
@media (max-width: 480px) {
  .app-header {
    padding: 14px;
  }
  .brand-name {
    font-size: 25px;
  }
  .brand-mark {
    width: 42px;
    height: 42px;
  }
  .status-context {
    display: none;
  }
}
</style>
