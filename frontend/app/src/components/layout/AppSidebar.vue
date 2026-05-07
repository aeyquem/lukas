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
  gap: var(--space-6);
  block-size: 100dvh;
  border-inline-end: 1px solid var(--color-border);
  background: color-mix(in srgb, var(--color-card) 80%, transparent);
  padding: var(--space-5);
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.sidebar-brand__logo {
  inline-size: 2.4rem;
  block-size: 2.4rem;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  object-fit: cover;
}

.sidebar-brand strong,
.sidebar-brand span {
  display: block;
}

.sidebar-brand strong {
  color: var(--color-primary-strong);
  font-size: 1.05rem;
  line-height: 1.1;
}

.sidebar-brand span {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-xs);
  font-weight: 700;
}

.sidebar-nav {
  display: grid;
  gap: var(--space-2);
}

.sidebar-option {
  display: grid;
  grid-template-columns: 2rem minmax(0, 1fr);
  align-items: center;
  gap: var(--space-3);
  min-block-size: 2.75rem;
  border: 1px solid transparent;
  border-radius: var(--radius-lg);
  background: transparent;
  color: var(--color-muted-foreground);
  padding: var(--space-2);
  text-align: start;
  transition:
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    color var(--motion-fast);
}

.sidebar-option span {
  display: inline-grid;
  place-items: center;
  inline-size: 2rem;
  block-size: 2rem;
  border-radius: var(--radius-md);
  background: var(--color-muted);
  color: var(--color-primary);
  font-size: var(--font-size-xs);
  font-weight: 800;
}

.sidebar-option strong {
  overflow: hidden;
  font-size: var(--font-size-sm);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-option:hover,
.sidebar-option:focus-visible,
.sidebar-option[aria-pressed="true"] {
  border-color: var(--color-border);
  background: var(--color-secondary);
  color: var(--color-primary-strong);
}

.sidebar-option[aria-pressed="true"] span {
  background: var(--color-primary);
  color: var(--color-primary-foreground);
}

@media (max-width: 900px) {
  .app-sidebar {
    position: static;
    block-size: auto;
    border-block-end: 1px solid var(--color-border);
    border-inline-end: 0;
  }

  .sidebar-nav {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 680px) {
  .app-sidebar {
    padding: var(--space-4);
  }

  .sidebar-nav {
    grid-template-columns: 1fr;
  }
}
</style>
