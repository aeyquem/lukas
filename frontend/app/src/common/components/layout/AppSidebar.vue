<script setup lang="ts">
import type { WorkspaceSection } from "@/common/workspace/sections";

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
  gap: var(--lu-space-6);
  block-size: 100dvh;
  border-inline-end: 1px solid var(--lu-color-border);
  background: var(--lu-color-surface);
  padding: var(--lu-space-5);
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: var(--lu-space-3);
}

.sidebar-brand__logo {
  inline-size: 2.5rem;
  block-size: 2.5rem;
  border-radius: var(--lu-radius-lg);
  box-shadow: var(--lu-shadow-sm);
  object-fit: cover;
}

.sidebar-brand strong,
.sidebar-brand span {
  display: block;
}

.sidebar-brand strong {
  font-size: var(--lu-font-size-md);
  line-height: var(--lu-line-tight);
}

.sidebar-brand span {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 700;
}

.sidebar-nav {
  display: grid;
  gap: var(--lu-space-2);
}

.sidebar-option {
  display: grid;
  grid-template-columns: var(--lu-size-icon) minmax(0, 1fr);
  align-items: center;
  gap: var(--lu-space-3);
  min-block-size: 2.75rem;
  border: 1px solid transparent;
  border-radius: var(--lu-radius-lg);
  background: transparent;
  color: var(--lu-color-text-muted);
  padding: var(--lu-space-2);
  text-align: start;
  transition:
    background-color var(--lu-transition-fast),
    border-color var(--lu-transition-fast),
    color var(--lu-transition-fast);
}

.sidebar-option span {
  display: inline-grid;
  place-items: center;
  inline-size: var(--lu-size-icon);
  block-size: var(--lu-size-icon);
  border-radius: var(--lu-radius-md);
  background: var(--lu-color-surface-muted);
  color: var(--lu-color-primary);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
}

.sidebar-option strong {
  overflow: hidden;
  font-size: var(--lu-font-size-sm);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-option:hover,
.sidebar-option:focus-visible,
.sidebar-option[aria-pressed="true"] {
  border-color: var(--lu-color-border);
  background: var(--lu-color-surface-muted);
  color: var(--lu-color-text);
}

.sidebar-option[aria-pressed="true"] span {
  background: var(--lu-color-primary);
  color: var(--lu-color-primary-contrast);
}

@media (max-width: 900px) {
  .app-sidebar {
    position: static;
    block-size: auto;
    border-block-end: 1px solid var(--lu-color-border);
    border-inline-end: 0;
  }

  .sidebar-nav {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 680px) {
  .app-sidebar {
    padding: var(--lu-space-4);
  }

  .sidebar-nav {
    grid-template-columns: 1fr;
  }
}
</style>
