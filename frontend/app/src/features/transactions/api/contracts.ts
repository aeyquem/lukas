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


export interface AccountTransactionPage {
  items: TransactionView[]
  next: { before_date: string; before_id: string } | null
}

export interface TransactionCursor {
  before_date: string
  before_id: string
}

export interface TransactionPage extends AccountTransactionPage {
  next: TransactionCursor | null
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

