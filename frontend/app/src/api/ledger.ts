import { getJson, postJson, putJson } from "@/api/client";
import type { LedgerMovement, MovementDirection } from "@/ledger/movements";

export type AccountType = "cash" | "bank" | "credit_card";
export type TransactionKind = "income" | "expense" | "adjustment";
export type AccountEffect = "increase" | "decrease";

export interface AccountDto {
  id: string;
  displayName: string;
  type: AccountType;
  currency: string;
  currentBalance: string;
}

export interface CreateAccountRequest {
  displayName: string;
  type: AccountType;
  currency: string;
  openingBalance?: string | null;
}

export interface TransactionDto {
  id: string;
  kind: TransactionKind;
  accountId: string;
  transactionDate: string;
  amount: string;
  currency: string;
  accountEffect: AccountEffect;
  description: string;
  categoryName?: string | null;
  notes?: string | null;
}

export interface MovementDto {
  id: string;
  transactionId: string;
  transactionDate: string;
  accountId: string;
  accountName: string;
  kind: TransactionKind;
  accountEffect: AccountEffect;
  direction: MovementDirection;
  originalAmount: string;
  originalCurrency: string;
  baseAmount: string;
  baseCurrency: string;
  description: string;
  categoryName?: string | null;
  notes?: string | null;
}

export interface CreateManualTransactionRequest {
  kind: TransactionKind;
  accountId: string;
  transactionDate: string;
  amount: string;
  currency: string;
  accountEffect: AccountEffect;
  description: string;
  categoryName?: string | null;
  notes?: string | null;
}

export interface UpdateManualTransactionRequest {
  id: string;
  kind: TransactionKind;
  accountId: string;
  transactionDate: string;
  amount: string;
  currency: string;
  accountEffect: AccountEffect;
  description: string;
  categoryName?: string | null;
  notes?: string | null;
}

export interface MonthlyCashflowDto {
  period: string;
  currency: string;
  incomeTotal: string;
  expenseTotal: string;
  netCashflow: string;
  accounts: AccountDto[];
}

export async function getAccounts(): Promise<AccountDto[]> {
  const result = await getJson<AccountDto[]>("/api/accounts");
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function createAccount(
  request: CreateAccountRequest,
): Promise<AccountDto> {
  const result = await postJson<AccountDto, CreateAccountRequest>(
    "/api/accounts",
    request,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function getTransactions(
  period: string,
): Promise<TransactionDto[]> {
  const result = await getJson<TransactionDto[]>(
    `/api/transactions?period=${encodeURIComponent(period)}`,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function createManualTransaction(
  request: CreateManualTransactionRequest,
): Promise<TransactionDto> {
  const result = await postJson<TransactionDto, CreateManualTransactionRequest>(
    "/api/transactions",
    request,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function updateManualTransaction(
  request: UpdateManualTransactionRequest,
): Promise<TransactionDto> {
  const { id, ...body } = request;
  const result = await putJson<TransactionDto, typeof body>(
    `/api/transactions/${encodeURIComponent(id)}`,
    body,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function getMovements(): Promise<LedgerMovement[]> {
  const result = await getJson<MovementDto[]>("/api/movements");
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value.map((movement) => ({
    id: movement.id,
    transactionId: movement.transactionId,
    date: movement.transactionDate,
    accountId: movement.accountId,
    accountName: movement.accountName,
    kind: movement.kind,
    accountEffect: movement.accountEffect,
    description: movement.description,
    direction: movement.direction,
    originalAmount: movement.originalAmount,
    originalCurrency: movement.originalCurrency,
    baseAmount: movement.baseAmount,
    baseCurrency: movement.baseCurrency,
    ...(movement.categoryName ? { categoryName: movement.categoryName } : {}),
    ...(movement.notes ? { notes: movement.notes } : {}),
  }));
}

export async function getMonthlyCashflow(
  period: string,
): Promise<MonthlyCashflowDto> {
  const result = await getJson<MonthlyCashflowDto>(
    `/api/cashflow/monthly?period=${encodeURIComponent(period)}`,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}
