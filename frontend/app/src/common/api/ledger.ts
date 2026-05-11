import { deleteJson, getJson, postJson, putJson } from "@/common/api/client";

export type AccountType = "cash" | "bank" | "credit_card";
export type TransactionKind = "income" | "expense" | "adjustment" | "transfer";
export type AccountEffect = "increase" | "decrease";
export type MovementDirection = "in" | "out";

export interface AccountDto {
  id: string;
  displayName: string;
  type: AccountType;
  currency: string;
  currentBalance: string;
}

export interface TagDto {
  id: string;
  name: string;
  normalizedName: string;
  color: string;
}

export interface CreateTagRequest {
  name: string;
  color: string;
}

export interface UpdateTagRequest {
  id: string;
  name: string;
  color: string;
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
  destinationAccountId?: string | null;
  transactionDate: string;
  amount: string;
  currency: string;
  accountEffect: AccountEffect;
  description: string;
  envelopeId?: string | null;
  envelopeName?: string | null;
  categoryName?: string | null;
  notes?: string | null;
  tags: TagDto[];
}

export interface MovementDto {
  id: string;
  transactionId: string;
  transactionDate: string;
  accountId: string;
  accountName: string;
  destinationAccountId?: string | null;
  kind: TransactionKind;
  accountEffect: AccountEffect;
  direction: MovementDirection;
  originalAmount: string;
  originalCurrency: string;
  baseAmount: string;
  baseCurrency: string;
  description: string;
  envelopeId?: string | null;
  envelopeName?: string | null;
  categoryName?: string | null;
  notes?: string | null;
  tags: TagDto[];
}

export type LedgerMovement = {
  id: string;
  transactionId: string;
  date: string;
  accountId: string;
  accountName: string;
  destinationAccountId?: string;
  kind: TransactionKind;
  accountEffect: AccountEffect;
  envelopeId?: string;
  envelopeName?: string;
  categoryName?: string;
  description: string;
  notes?: string;
  tags: TagDto[];
  direction: MovementDirection;
  originalAmount: string;
  originalCurrency: string;
  baseAmount: string;
  baseCurrency: string;
};

export interface CreateManualTransactionRequest {
  kind: TransactionKind;
  accountId: string;
  destinationAccountId?: string | null;
  transactionDate: string;
  amount: string;
  currency: string;
  accountEffect: AccountEffect;
  description: string;
  envelopeId?: string | null;
  categoryName?: string | null;
  notes?: string | null;
  tags?: string[] | null;
}

export interface UpdateManualTransactionRequest {
  id: string;
  kind: TransactionKind;
  accountId: string;
  destinationAccountId?: string | null;
  transactionDate: string;
  amount: string;
  currency: string;
  accountEffect: AccountEffect;
  description: string;
  envelopeId?: string | null;
  categoryName?: string | null;
  notes?: string | null;
  tags?: string[] | null;
}

export interface MonthlyCashflowDto {
  period: string;
  currency: string;
  incomeTotal: string;
  expenseTotal: string;
  netCashflow: string;
  accounts: AccountDto[];
}

export interface MonthlyTagReportDto {
  period: string;
  currency: string;
  tags: TagReportRowDto[];
}

export interface TagReportRowDto {
  tag: TagDto;
  incomeTotal: string;
  expenseTotal: string;
  netTotal: string;
  transactionCount: number;
}

export interface BudgetTreeDto {
  period: string;
  categories: BudgetCategoryDto[];
  subcategories: BudgetSubcategoryDto[];
  envelopes: BudgetEnvelopeDto[];
  goals: BudgetGoalDto[];
}

export interface BudgetCategoryDto {
  id: string;
  name: string;
  sortOrder: number;
}

export interface BudgetSubcategoryDto {
  id: string;
  categoryId: string;
  name: string;
  sortOrder: number;
}

export interface BudgetEnvelopeDto {
  id: string;
  subcategoryId: string;
  name: string;
  assignedAmount: string;
  activityAmount: string;
  remainingAmount: string;
  sortOrder: number;
}

export type BudgetGoalStatus = "active" | "completed" | "archived";

export interface BudgetGoalDto {
  id: string;
  envelopeId: string;
  targetAmount: string;
  deadlineDate: string;
  status: BudgetGoalStatus;
}

export async function getAccounts(): Promise<AccountDto[]> {
  const result = await getJson<AccountDto[]>("/api/accounts");
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function getTags(): Promise<TagDto[]> {
  const result = await getJson<TagDto[]>("/api/tags");
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function createTag(request: CreateTagRequest): Promise<TagDto> {
  const result = await postJson<TagDto, CreateTagRequest>("/api/tags", request);
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function updateTag(request: UpdateTagRequest): Promise<TagDto> {
  const { id, ...body } = request;
  const result = await putJson<TagDto, typeof body>(
    `/api/tags/${encodeURIComponent(id)}`,
    body,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function deleteTag(id: string): Promise<true> {
  const result = await deleteJson(`/api/tags/${encodeURIComponent(id)}`);
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
    ...(movement.destinationAccountId
      ? { destinationAccountId: movement.destinationAccountId }
      : {}),
    kind: movement.kind,
    accountEffect: movement.accountEffect,
    description: movement.description,
    direction: movement.direction,
    originalAmount: movement.originalAmount,
    originalCurrency: movement.originalCurrency,
    baseAmount: movement.baseAmount,
    baseCurrency: movement.baseCurrency,
    ...(movement.envelopeId ? { envelopeId: movement.envelopeId } : {}),
    ...(movement.envelopeName ? { envelopeName: movement.envelopeName } : {}),
    ...(movement.categoryName ? { categoryName: movement.categoryName } : {}),
    ...(movement.notes ? { notes: movement.notes } : {}),
    tags: movement.tags ?? [],
  }));
}

export async function getBudgetTree(period: string): Promise<BudgetTreeDto> {
  const result = await getJson<BudgetTreeDto>(
    `/api/budget?period=${encodeURIComponent(period)}`,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}

export async function saveBudgetTree(
  period: string,
  request: BudgetTreeDto,
): Promise<BudgetTreeDto> {
  const result = await putJson<BudgetTreeDto, BudgetTreeDto>(
    `/api/budget?period=${encodeURIComponent(period)}`,
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

export async function getMonthlyTagReport(
  period: string,
): Promise<MonthlyTagReportDto> {
  const result = await getJson<MonthlyTagReportDto>(
    `/api/reports/tags?period=${encodeURIComponent(period)}`,
  );
  if (!result.ok) {
    throw new Error(result.error.title);
  }

  return result.value;
}
