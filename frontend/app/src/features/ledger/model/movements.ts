import type { MovementDirection } from "@/common/api/ledger";

export type { LedgerMovement, MovementDirection } from "@/common/api/ledger";

export function directionLabel(direction: MovementDirection): string {
  return direction === "in" ? "Entrada" : "Salida";
}
