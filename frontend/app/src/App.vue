<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { activeTheme, setTheme, themeNotice, themes } from './app/theme'
import { currentPeriodInSantiago } from './components/budget/budget-model'

const now = ref(new Date())
const route = useRoute()
const router = useRouter()
const dashboardPeriod = computed(() => {
  const period = route.query.dashboardPeriod
  return typeof period === 'string' && /^\d{4}-(0[1-9]|1[0-2])$/.test(period) ? period : currentPeriodInSantiago(now.value)
})
const monthLabel = computed(() => new Intl.DateTimeFormat('es-CL', {
  month: 'long', year: 'numeric', timeZone: 'UTC',
}).format(new Date(`${dashboardPeriod.value}-01T12:00:00Z`)))
function changeDashboardPeriod(value: string): void {
  if (!/^\d{4}-\d{2}$/.test(value)) return
  void router.replace({ name: 'home', query: { dashboardPeriod: value } })
}
let clock: ReturnType<typeof setInterval> | undefined
onMounted(() => { clock = setInterval(() => { now.value = new Date() }, 60_000) })
onUnmounted(() => clearInterval(clock))
</script>

<template>
  <div class="app-shell">
    <a class="skip-link" href="#main-content">Ir al contenido</a>
    <header class="app-header island">
      <RouterLink class="brand" :to="{ name: 'home' }" aria-label="Lukas, inicio">
        <span class="brand-mark" aria-hidden="true">L<span class="brand-leaf"></span></span>
        <span class="brand-name">lukas<span class="brand-dot">.</span></span>
      </RouterLink>

      <RouterLink class="current-period" :to="{ name: 'home', query: { dashboardPeriod } }" aria-label="Contexto del período del panel">
        <span class="period-icon" aria-hidden="true">▦</span>
        <span><small>Período del panel</small><span>{{ monthLabel }}</span></span>
      </RouterLink>
      <label class="period-picker" for="dashboard-period">Cambiar período
        <input id="dashboard-period" type="month" :value="dashboardPeriod" @change="changeDashboardPeriod(($event.target as HTMLInputElement).value)">
      </label>

      <nav aria-label="Navegación principal">
        <RouterLink :to="{ name: 'home', query: { dashboardPeriod } }">Panel</RouterLink>
        <RouterLink :to="{ name: 'budget' }">Presupuesto</RouterLink>
        <RouterLink :to="{ name: 'accounts' }">Cuentas</RouterLink>
        <RouterLink :to="{ name: 'transactions' }">Transacciones</RouterLink>
        <RouterLink :to="{ name: 'payees' }">Payees</RouterLink>
        <RouterLink :to="{ name: 'projections' }">Proyecciones</RouterLink>
        <RouterLink :to="{ name: 'objectives' }">Objetivos</RouterLink>
      </nav>

      <details class="appearance-menu">
        <summary aria-label="Menú de usuario" title="Menú de usuario"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="8" r="3.4" /><path d="M5.5 20c.7-3.3 3-5 6.5-5s5.8 1.7 6.5 5" /></svg></summary>
        <div class="appearance-panel">
          <p class="eyebrow">Espacio personal</p>
          <p class="profile-context">Instancia owner-only · acceso local</p>
          <label for="theme-select">Apariencia</label>
          <select id="theme-select" :value="activeTheme" @change="setTheme(($event.target as HTMLSelectElement).value)">
            <option v-for="theme in themes" :key="theme.id" :value="theme.id">{{ theme.label }}</option>
          </select>
          <p v-if="themeNotice" role="status">{{ themeNotice }}</p>
        </div>
      </details>
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
.current-period {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-inline: auto;
  white-space: nowrap;
  font-size: 13px;
  color: inherit;
  text-decoration: none;
}
.period-picker { display: grid; gap: 3px; color: var(--muted); font-size: 10px; white-space: nowrap; }
.period-picker input { width: 132px; min-height: 34px; padding: 4px 8px; border: 1px solid var(--border-strong); border-radius: var(--control-radius); color: var(--text); background: var(--surface-raised); font: inherit; font-size: 12px; }
.current-period > span:last-child {
  display: grid;
  gap: 3px;
}
.current-period small {
  color: var(--muted);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: .1em;
}
.period-icon {
  color: var(--accent);
  font-size: 23px;
}
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
.appearance-menu {
  flex-shrink: 0;
}
.appearance-menu summary {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 1px solid var(--border-strong);
  background: var(--accent-soft);
  color: var(--accent);
  cursor: pointer;
  font-weight: 650;
  list-style: none;
}
.appearance-menu summary svg { width: 22px; height: 22px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; }
.appearance-menu summary::-webkit-details-marker {
  display: none;
}
.appearance-panel {
  position: absolute;
  right: 16px;
  top: calc(100% + 10px);
  display: grid;
  gap: 12px;
  width: min(260px, calc(100vw - 60px));
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: var(--island-radius);
  background: var(--surface);
  box-shadow: var(--shadow-floating);
}
.appearance-panel p {
  margin: 0;
  font-size: 12px;
}
.appearance-panel .profile-context { color: var(--muted); line-height: 1.45; }
.appearance-panel label {
  font-weight: 650;
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
  .current-period {
    margin-left: auto;
  }
  nav {
    order: 4;
    width: 100%;
    justify-content: center;
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
  .current-period {
    font-size: 11px;
    margin-right: 0;
  }
  .period-picker { order: 3; }
  .period-picker input { width: 118px; }
  .period-icon {
    display: none;
  }
  nav {
    gap: 2px;
  }
  nav a {
    padding: 10px 8px;
    font-size: 11px;
  }
  .status-context {
    display: none;
  }
}
</style>
