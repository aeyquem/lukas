<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
const props = defineProps<{ dashboardPeriod: string }>()
const router = useRouter()
const monthLabel = computed(() => new Intl.DateTimeFormat('es-CL', {
  month: 'long', year: 'numeric', timeZone: 'UTC',
}).format(new Date(`${props.dashboardPeriod}-01T12:00:00Z`)))
function changeDashboardPeriod(value: string): void {
  if (!/^\d{4}-\d{2}$/.test(value)) return
  void router.replace({ name: 'home', query: { dashboardPeriod: value } })
}
</script>

<template>
  <RouterLink class="current-period" :to="{ name: 'home', query: { dashboardPeriod } }" aria-label="Contexto del período del panel">
    <span class="period-icon" aria-hidden="true">▦</span>
    <span><small>Período del panel</small><span>{{ monthLabel }}</span></span>
  </RouterLink>
  <label class="period-picker" for="dashboard-period">Cambiar período
    <input id="dashboard-period" type="month" :value="dashboardPeriod" @change="changeDashboardPeriod(($event.target as HTMLInputElement).value)">
  </label>
</template>

<style scoped>
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
@media (max-width: 1050px) {
  .current-period { margin-left: auto; }
}
@media (max-width: 480px) {
  .current-period { font-size: 11px; margin-right: 0; }
  .period-picker { order: 3; }
  .period-picker input { width: 118px; }
  .period-icon { display: none; }
}
</style>
