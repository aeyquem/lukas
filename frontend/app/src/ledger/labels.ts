import type { AccountType, TransactionKind } from "@/api/ledger";

const accountTypeLabels: Record<AccountType, string> = {
  cash: "Efectivo",
  bank: "Banco",
  credit_card: "Tarjeta",
};

const transactionKindLabels: Record<TransactionKind, string> = {
  income: "Ingreso",
  expense: "Gasto",
  adjustment: "Ajuste",
  transfer: "Transferencia",
};

export function accountTypeLabel(type: AccountType): string {
  return accountTypeLabels[type];
}

export function transactionKindLabel(kind: TransactionKind): string {
  return transactionKindLabels[kind];
}
