<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { themes, useAppearanceStore } from '../theme-store'
const appearance = useAppearanceStore()
const { activeTheme, themeNotice } = storeToRefs(appearance)
const { setTheme } = appearance
</script>

<template>
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
</template>

<style scoped>
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
</style>
