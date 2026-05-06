export type WorkspaceSection = "accounts" | "budget" | "reports" | "projections";

const sectionTitles: Record<WorkspaceSection, string> = {
  accounts: "Cuentas",
  budget: "Presupuesto",
  reports: "Reportes",
  projections: "Proyecciones",
};

const sectionEyebrows: Record<WorkspaceSection, string> = {
  accounts: "Libro manual y saldos",
  budget: "Plan mensual",
  reports: "Resumen financiero",
  projections: "Escenarios",
};

export function workspaceSectionTitle(section: WorkspaceSection): string {
  return sectionTitles[section];
}

export function workspaceSectionEyebrow(section: WorkspaceSection): string {
  return sectionEyebrows[section];
}
