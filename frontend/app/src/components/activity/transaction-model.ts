import { assertBusinessDate } from '../../shared/formatting/business-date'

export type TransactionKind = 'income' | 'expense' | 'refund' | 'transfer'
export type OpeningTreatment = '' | 'apply_effect' | 'preserve_balance'

export interface OpeningView {
  business_date: string
}

export interface WalletView {
  id: string
  currency: string
  openings: OpeningView[]
}

export interface AccountView {
  id: string
  name: string
  wallets: WalletView[]
}

export interface EffectView {
  wallet_id: string
  currency: string
  role: 'single' | 'source' | 'destination'
  amount: string
  opening_treatment: string | null
}

export interface TransactionView {
  id: string
  kind: TransactionKind
  business_date: string
  category_id: string | null
  source_expense_id: string | null
  payee: string | null
  note: string | null
  confirmed_clp_amount: string | null
  conversion_evidence: unknown
  revision: number
  effects: EffectView[]
}

export interface TransactionDraft {
  kind: TransactionKind
  businessDate: string
  walletId: string
  amount: string
  openingTreatment: OpeningTreatment
  sourceWalletId: string
  sourceAmount: string
  sourceOpeningTreatment: OpeningTreatment
  destinationWalletId: string
  destinationAmount: string
  destinationOpeningTreatment: OpeningTreatment
  categoryId: string
  sourceExpenseId: string
  payee: string
  note: string
  confirmedClpAmount: string
  conversionEvidence: string
  reason: string
}

export interface TransactionCategory {
  id: string
  name: string
  archived: boolean
}

export interface TransactionRequest {
  kind: TransactionKind
  business_date: string
  wallet_id?: string
  amount?: string
  opening_treatment?: Exclude<OpeningTreatment, ''>
  source_wallet_id?: string
  source_amount?: string
  source_opening_treatment?: Exclude<OpeningTreatment, ''>
  destination_wallet_id?: string
  destination_amount?: string
  destination_opening_treatment?: Exclude<OpeningTreatment, ''>
  category_id?: string
  source_expense_id?: string
  payee?: string
  note?: string
  confirmed_clp_amount?: string
  conversion_evidence?: unknown
  expected_revision?: number
  reason?: string
}

export function createTransactionDraft(today: string): TransactionDraft {
  return {
    kind: 'expense',
    businessDate: assertBusinessDate(today),
    walletId: '',
    amount: '',
    openingTreatment: '',
    sourceWalletId: '',
    sourceAmount: '',
    sourceOpeningTreatment: '',
    destinationWalletId: '',
    destinationAmount: '',
    destinationOpeningTreatment: '',
    categoryId: '',
    sourceExpenseId: '',
    payee: '',
    note: '',
    confirmedClpAmount: '',
    conversionEvidence: '',
    reason: '',
  }
}

export function latestOpeningDate(wallet: WalletView | undefined): string | undefined {
  return wallet?.openings
    .map((opening) => assertBusinessDate(opening.business_date))
    .sort()
    .at(-1)
}

export function requiresOpeningTreatment(date: string, wallet: WalletView | undefined): boolean {
  const latestOpening = latestOpeningDate(wallet)
  return latestOpening !== undefined && assertBusinessDate(date) <= latestOpening
}

export function positiveMagnitude(signedAmount: string): string {
  return signedAmount.startsWith('-') ? signedAmount.slice(1) : signedAmount
}

export function draftFromTransaction(
  transaction: TransactionView,
  accounts: AccountView[],
): TransactionDraft {
  const draft = createTransactionDraft(transaction.business_date)
  draft.kind = transaction.kind
  draft.businessDate = transaction.business_date
  draft.categoryId = transaction.category_id ?? ''
  draft.sourceExpenseId = transaction.source_expense_id ?? ''
  draft.payee = transaction.payee ?? ''
  draft.note = transaction.note ?? ''
  draft.confirmedClpAmount = transaction.confirmed_clp_amount ?? ''
  draft.conversionEvidence =
    transaction.conversion_evidence === null
      ? ''
      : JSON.stringify(transaction.conversion_evidence, null, 2)

  if (transaction.kind === 'transfer') {
    const source = transaction.effects.find((effect) => effect.role === 'source')
    const destination = transaction.effects.find((effect) => effect.role === 'destination')
    if (source) {
      draft.sourceWalletId = source.wallet_id
      draft.sourceAmount = positiveMagnitude(source.amount)
      draft.sourceOpeningTreatment = openingTreatment(source.opening_treatment)
    }
    if (destination) {
      draft.destinationWalletId = destination.wallet_id
      draft.destinationAmount = positiveMagnitude(destination.amount)
      draft.destinationOpeningTreatment = openingTreatment(destination.opening_treatment)
    }
  } else {
    const single = transaction.effects.find((effect) => effect.role === 'single')
    if (single) {
      draft.walletId = single.wallet_id
      draft.amount = positiveMagnitude(single.amount)
      draft.openingTreatment = openingTreatment(single.opening_treatment)
    }
  }

  // Ensure the stored wallet identifiers still exist in the current account view.
  const walletIds = new Set(accounts.flatMap((account) => account.wallets.map((wallet) => wallet.id)))
  if (!walletIds.has(draft.walletId)) draft.walletId = ''
  if (!walletIds.has(draft.sourceWalletId)) draft.sourceWalletId = ''
  if (!walletIds.has(draft.destinationWalletId)) draft.destinationWalletId = ''
  return draft
}

function openingTreatment(value: string | null): OpeningTreatment {
  return value === 'apply_effect' || value === 'preserve_balance' ? value : ''
}

/** Build one complete wire request while leaving exact monetary strings intact. */
export function buildTransactionRequest(
  draft: TransactionDraft,
  expectedRevision?: number,
): TransactionRequest {
  const body: TransactionRequest = {
    kind: draft.kind,
    business_date: assertBusinessDate(draft.businessDate),
  }

  if (draft.kind === 'transfer') {
    body.source_wallet_id = draft.sourceWalletId
    body.source_amount = draft.sourceAmount
    body.destination_wallet_id = draft.destinationWalletId
    body.destination_amount = draft.destinationAmount
    if (draft.sourceOpeningTreatment) {
      body.source_opening_treatment = draft.sourceOpeningTreatment
    }
    if (draft.destinationOpeningTreatment) {
      body.destination_opening_treatment = draft.destinationOpeningTreatment
    }
    if (draft.conversionEvidence.trim()) {
      body.conversion_evidence = JSON.parse(draft.conversionEvidence) as unknown
    }
  } else {
    body.wallet_id = draft.walletId
    body.amount = draft.amount
    if (draft.openingTreatment) body.opening_treatment = draft.openingTreatment
    if (draft.kind === 'income' && draft.categoryId) body.category_id = draft.categoryId
    if (draft.kind === 'expense' && draft.categoryId) body.category_id = draft.categoryId
    if (draft.kind === 'refund') {
      if (draft.sourceExpenseId) body.source_expense_id = draft.sourceExpenseId
      // A linked refund inherits its source category and the backend rejects a mismatch.
      else if (draft.categoryId) body.category_id = draft.categoryId
    }
    if (draft.kind === 'expense' && draft.confirmedClpAmount.trim()) {
      body.confirmed_clp_amount = draft.confirmedClpAmount
    }
  }

  if (draft.payee.trim()) body.payee = draft.payee.trim()
  if (draft.note.trim()) body.note = draft.note.trim()

  if (expectedRevision !== undefined) {
    body.expected_revision = expectedRevision
    body.reason = draft.reason.trim()
  }
  return body
}

