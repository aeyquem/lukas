<script setup lang="ts">
import type { WorkspaceSection } from "@/workspace/sections";

const props = defineProps<{
  activeSection: WorkspaceSection;
}>();

const emit = defineEmits<{
  "select-section": [section: WorkspaceSection];
}>();

const sections: Array<{
  key: WorkspaceSection;
  marker: string;
  label: string;
}> = [
  { key: "accounts", marker: "C", label: "Cuentas" },
  { key: "movements", marker: "M", label: "Movimientos" },
  { key: "budget", marker: "P", label: "Presupuesto" },
  { key: "goals", marker: "G", label: "Metas" },
  { key: "reports", marker: "R", label: "Reportes" },
  { key: "projections", marker: "Y", label: "Proyecciones" },
];

function selectSection(section: WorkspaceSection) {
  emit("select-section", section);
}
</script>

<template>
  <aside class="app-sidebar" aria-label="Navegacion principal">
    <div class="sidebar-brand">
      <img
        alt=""
        aria-hidden="true"
        class="sidebar-brand__logo"
        src="/assets/lukas-icon.png"
      />
      <div>
        <strong>Lukas</strong>
        <span>Finanzas personales</span>
      </div>
    </div>

    <nav class="sidebar-nav">
      <button
        v-for="section in sections"
        :key="section.key"
        :aria-pressed="props.activeSection === section.key"
        class="sidebar-option"
        type="button"
        @click="selectSection(section.key)"
      >
        <span aria-hidden="true">{{ section.marker }}</span>
        <strong>{{ section.label }}</strong>
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.app-sidebar {
  position: sticky;
  inset-block-start: 0;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  block-size: 100dvh;
  border-inline-end: 1px solid var(--p-content-border-color);
  background: var(--p-content-background);
  padding: 1.25rem;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.sidebar-brand__logo {
  inline-size: 2.4rem;
  block-size: 2.4rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgb(0 0 0 / 0.08);
  object-fit: cover;
}

.sidebar-brand strong,
.sidebar-brand span {
  display: block;
}

.sidebar-brand strong {
  font-size: 1.05rem;
  line-height: 1.1;
}

.sidebar-brand span {
  color: var(--p-text-muted-color);
  font-size: 0.75rem;
  font-weight: 700;
}

.sidebar-nav {
  display: grid;
  gap: 0.5rem;
}

.sidebar-option {
  display: grid;
  grid-template-columns: 2rem minmax(0, 1fr);
  align-items: center;
  gap: 0.75rem;
  min-block-size: 2.75rem;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  color: var(--p-text-muted-color);
  padding: 0.5rem;
  text-align: start;
  transition:
    background-color 180ms ease,
    border-color 180ms ease,
    color 180ms ease;
}

.sidebar-option span {
  display: inline-grid;
  place-items: center;
  inline-size: 2rem;
  block-size: 2rem;
  border-radius: 6px;
  background: var(--p-content-hover-background);
  color: var(--p-primary-color);
  font-size: 0.75rem;
  font-weight: 800;
}

.sidebar-option strong {
  overflow: hidden;
  font-size: 0.875rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-option:hover,
.sidebar-option:focus-visible,
.sidebar-option[aria-pressed="true"] {
  border-color: var(--p-content-border-color);
  background: var(--p-content-hover-background);
  color: var(--p-text-color);
}

.sidebar-option[aria-pressed="true"] span {
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color);
}

@media (max-width: 900px) {
  .app-sidebar {
    position: static;
    block-size: auto;
    border-block-end: 1px solid var(--p-content-border-color);
    border-inline-end: 0;
  }

  .sidebar-nav {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 680px) {
  .app-sidebar {
    padding: 1rem;
  }

  .sidebar-nav {
    grid-template-columns: 1fr;
  }
}
</style>
