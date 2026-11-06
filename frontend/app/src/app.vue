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
        <span class="brand-mark" aria-hidden="true">L<span class="brand-leaf"></span></span>
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
  position: relative;
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 14px;
  color: var(--on-accent);
  background: var(--accent);
  font-size: 27px;
  font-weight: 600;
  font-family: var(--font-display);
  box-shadow: inset 0 1px 0 var(--edge-light);
}
.brand-leaf {
  position: absolute;
  width: 9px;
  height: 13px;
  border-radius: 9px 0 9px 0;
  background: var(--accent-soft);
  top: 8px;
  right: 9px;
  transform: rotate(18deg);
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
    width: 36px;
    height: 36px;
  }
  .status-context {
    display: none;
  }
}
</style>
