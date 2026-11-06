import { afterEach, describe, expect, it, vi } from 'vitest'
import { isValidMoney, newDraft, newWallet, validateAccountDraft, signOf, todayInSantiago } from '../src/features/accounts/account-draft'
import { setDraftAccountType } from '../src/features/accounts/account-model'

describe('account creation draft rules', () => {
  it('preserves currencies and opening dates when changing account type', () => {
    const draft = newDraft()
    draft.name = 'Household'
    draft.account_type = 'cash'
    draft.wallets.push({ ...newWallet(), currency: 'USD', opening_date: '2026-01-01' })
    setDraftAccountType(draft, 'bank')
    expect(draft.wallets[1]?.opening_date).toBe('2026-01-01')
    expect(validateAccountDraft(draft).wallets).toBeDefined()
    setDraftAccountType(draft, 'card')
    expect(validateAccountDraft(draft)).toEqual({})
  })

  it('requires a signed CLP basis for eligible foreign debt', () => {
    const draft = newDraft()
    draft.name = 'Card'
    Object.assign(draft.wallets[0]!, { currency: 'USD', opening_amount: '-10', liability_clp_basis: '100' })
    expect(validateAccountDraft(draft)['wallets[0].liability_clp_basis']).toBeDefined()
    draft.wallets[0]!.liability_clp_basis = '-100'
    expect(validateAccountDraft(draft)).toEqual({})
  })

  it('rejects lossy decimal syntax and values beyond the backend envelope', () => {
    for (const invalid of ['1e2', '1,000', ' 1', 'NaN', '79228162514264337593543950336', '0.12345678901234567890123456789']) {
      expect(isValidMoney(invalid)).toBe(false)
    }
    expect(isValidMoney('79228162514264337593543950335')).toBe(true)
  })
})


afterEach(() => vi.useRealTimers())

it('distinguishes negative zero and preserves the Santiago business date in independent drafts', () => {
  vi.useFakeTimers()
  vi.setSystemTime(new Date('2026-06-01T01:00:00Z'))
  expect(todayInSantiago()).toBe('2026-05-31')
  expect(signOf('-0.00')).toBe(0)
  expect(signOf('-0.01')).toBe(-1)
  const first = newDraft()
  const second = newDraft()
  first.wallets[0]!.opening_amount = '100'
  expect(second.wallets[0]!.opening_amount).toBe('0')
  expect(second.wallets[0]!.opening_date).toBe('2026-05-31')
})
