import type { AccountEffect, TransactionKind } from "@/api/ledger";

export type MovementDirection = "in" | "out";

export type LedgerMovement = {
  id: string;
  transactionId: string;
  date: string;
  accountId: string;
  accountName: string;
  kind: TransactionKind;
  accountEffect: AccountEffect;
  categoryName?: string;
  description: string;
  notes?: string;
  direction: MovementDirection;
  originalAmount: string;
  originalCurrency: string;
  baseAmount: string;
  baseCurrency: string;
};

export function directionLabel(direction: MovementDirection): string {
  return direction === "in" ? "Entrada" : "Salida";
}
