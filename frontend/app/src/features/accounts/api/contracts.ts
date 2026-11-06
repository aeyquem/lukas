import type { AccountType } from '../account-model'

export interface OpeningView {
  id: string
  amount: string
  business_date: string
  revision: number
  liability_clp_basis: string | null
  liability_basis_provenance: string | null
  liability_basis_revision: number | null
}

export interface WalletView {
  id: string
  currency: string
  balance: string
  openings: OpeningView[]
}

export interface AccountView {
  id: string
  name: string
  account_type: AccountType
  base_currency: string
  budget_eligible: boolean
  revision: number
  wallets: WalletView[]
}

export type { AccountDraft, WalletDraft } from '../account-draft'

export interface MutationReceipt {
  resource_id: string
  revision: number
}

export type { AccountTransactionPage } from '../../transactions/api/contracts'
