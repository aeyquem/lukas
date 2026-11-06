import { describe, expect, it } from 'vitest'
import {
  buildTransactionRequest,
  createTransactionDraft,
  draftFromTransaction,
  positiveMagnitude,
  requiresOpeningTreatment,
  type AccountView,
  type TransactionView,
} from '../src/components/activity/transaction-model'

const accounts: AccountView[] = [
  {
    id: 'account-1',
    name: 'Daily',
    wallets: [
      { id: 'wallet-clp', currency: 'CLP', openings: [{ business_date: '2026-03-01' }] },
      { id: 'wallet-usd', currency: 'USD', openings: [{ business_date: '2026-03-01' }] },
    ],
  },
]

describe('transaction form model', () => {
  it('requires an explicit opening choice before and on the opening date', () => {
    const wallet = accounts[0]!.wallets[0]
    expect(requiresOpeningTreatment('2026-02-28', wallet)).toBe(true)
    expect(requiresOpeningTreatment('2026-03-01', wallet)).toBe(true)
    expect(requiresOpeningTreatment('2026-03-02', wallet)).toBe(false)
  })

  it('checks the latest dated opening when a wallet has multiple opening entries', () => {
    const wallet = {
      id: 'wallet-multiple-openings',
      currency: 'CLP',
      openings: [{ business_date: '2026-03-01' }, { business_date: '2026-05-01' }],
    }
    expect(requiresOpeningTreatment('2026-04-30', wallet)).toBe(true)
    expect(requiresOpeningTreatment('2026-05-01', wallet)).toBe(true)
    expect(requiresOpeningTreatment('2026-05-02', wallet)).toBe(false)
  })

  it('keeps exact decimal strings when building the write request', () => {
    const draft = createTransactionDraft('2026-03-01')
    draft.kind = 'expense'
    draft.walletId = 'wallet-clp'
    draft.amount = '12345678901234567890.0001'
    draft.categoryId = 'category-1'
    draft.openingTreatment = 'preserve_balance'

    expect(buildTransactionRequest(draft)).toEqual({
      kind: 'expense',
      business_date: '2026-03-01',
      wallet_id: 'wallet-clp',
      amount: '12345678901234567890.0001',
      opening_treatment: 'preserve_balance',
      category_id: 'category-1',
    })
  })

  it('maps signed transfer effects back to positive amount fields for editing', () => {
    const transaction: TransactionView = {
      id: 'transaction-1',
      kind: 'transfer',
      business_date: '2026-03-02',
      category_id: null,
      source_expense_id: null,
      payee: null,
      note: 'Moved savings',
      confirmed_clp_amount: null,
      conversion_evidence: { reference: 'statement-4' },
      revision: 3,
      effects: [
        {
          wallet_id: 'wallet-clp',
          currency: 'CLP',
          role: 'source',
          amount: '-15000',
          opening_treatment: null,
        },
        {
          wallet_id: 'wallet-usd',
          currency: 'USD',
          role: 'destination',
          amount: '16.25',
          opening_treatment: 'apply_effect',
        },
      ],
    }

    const draft = draftFromTransaction(transaction, accounts)
    expect(draft.sourceWalletId).toBe('wallet-clp')
    expect(draft.sourceAmount).toBe('15000')
    expect(draft.destinationWalletId).toBe('wallet-usd')
    expect(draft.destinationAmount).toBe('16.25')
    expect(draft.destinationOpeningTreatment).toBe('apply_effect')
    expect(draft.conversionEvidence).toContain('statement-4')
  })

  it('keeps a linked refund attached to its source and inherits its category', () => {
    const draft = createTransactionDraft('2026-03-03')
    draft.kind = 'refund'
    draft.walletId = 'wallet-clp'
    draft.amount = '3500.25'
    draft.categoryId = 'source-category'
    draft.sourceExpenseId = 'expense-1'

    expect(buildTransactionRequest(draft, 2)).toEqual({
      kind: 'refund',
      business_date: '2026-03-03',
      wallet_id: 'wallet-clp',
      amount: '3500.25',
      source_expense_id: 'expense-1',
      expected_revision: 2,
      reason: '',
    })
  })

  it('preserves both exact transfer amounts and JSON conversion evidence', () => {
    const draft = createTransactionDraft('2026-03-04')
    draft.kind = 'transfer'
    draft.sourceWalletId = 'wallet-usd'
    draft.sourceAmount = '100000000000000.001'
    draft.destinationWalletId = 'wallet-clp'
    draft.destinationAmount = '93625001'
    draft.conversionEvidence = '{"source":"statement","rate":"936.25"}'

    expect(buildTransactionRequest(draft)).toEqual({
      kind: 'transfer',
      business_date: '2026-03-04',
      source_wallet_id: 'wallet-usd',
      source_amount: '100000000000000.001',
      destination_wallet_id: 'wallet-clp',
      destination_amount: '93625001',
      conversion_evidence: { source: 'statement', rate: '936.25' },
    })
  })

  it('removes only the sign from a signed amount string', () => {
    expect(positiveMagnitude('-0.000123456789')).toBe('0.000123456789')
    expect(positiveMagnitude('100000000000000000000.05')).toBe('100000000000000000000.05')
  })
})
