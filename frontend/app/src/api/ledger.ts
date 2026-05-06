import { getJson, postJson } from "./client";

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
