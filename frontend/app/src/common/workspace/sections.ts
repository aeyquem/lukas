export type WorkspaceSection =
  | "accounts"
  | "movements"
  | "budget"
  | "goals"
  | "reports"
  | "projections";

const sectionTitles: Record<WorkspaceSection, string> = {
  accounts: "Cuentas",
  movements: "Movimientos",
  budget: "Presupuesto",
  goals: "Metas",
  reports: "Reportes",
  projections: "Proyecciones",
};

const sectionEyebrows: Record<WorkspaceSection, string> = {
  accounts: "Libro manual y saldos",
  movements: "Registro de actividad",
  budget: "Plan mensual",
  goals: "Objetivos financieros",
  reports: "Resumen financiero",
  projections: "Escenarios",
};

export function workspaceSectionTitle(section: WorkspaceSection): string {
  return sectionTitles[section];
}

export function workspaceSectionEyebrow(section: WorkspaceSection): string {
  return sectionEyebrows[section];
}
