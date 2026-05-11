export type WorkspaceSection =
  | "accounts"
  | "movements"
  | "tags"
  | "budget"
  | "goals"
  | "reports"
  | "projections";

const sectionTitles: Record<WorkspaceSection, string> = {
  accounts: "Cuentas",
  movements: "Movimientos",
  tags: "Etiquetas",
  budget: "Presupuesto",
  goals: "Metas",
  reports: "Reportes",
  projections: "Proyecciones",
};

const sectionEyebrows: Record<WorkspaceSection, string> = {
  accounts: "Libro manual y saldos",
  movements: "Registro de actividad",
  tags: "Clasificación transversal",
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
