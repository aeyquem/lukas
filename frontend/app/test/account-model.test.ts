import { createSSRApp } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { renderToString } from 'vue/server-renderer'
import { describe, expect, it, vi } from 'vitest'
import AccountsView from '../src/views/accounts-view.vue'
import {
  canAddCurrencyWallet,
  setDraftAccountType,
  walletCountError,
} from '../src/features/accounts/account-model'

vi.mock('../src/shared/api/client', async (importOriginal) => ({
  ...await importOriginal<typeof import('../src/shared/api/client')>(),
  apiRequest: vi.fn().mockResolvedValue([]),
}))

function accountApp() {
  const app = createSSRApp(AccountsView)
  const pinia = createPinia()
  app.use(pinia)
  app.use(PiniaColada, { pinia })
  return app
}

describe('account currency wallet rules', () => {
  it('requires one currency for bank, savings, and investment accounts', () => {
    for (const accountType of ['bank', 'savings', 'investment'] as const) {
      expect(walletCountError(accountType, 0)).toBe('at_least_one_required')
      expect(walletCountError(accountType, 1)).toBeUndefined()
      expect(walletCountError(accountType, 2))
        .toBe('multiple_currencies_unsupported_for_account_type')
      expect(canAddCurrencyWallet(accountType)).toBe(false)
    }
  })

  it('allows card and cash accounts to track multiple currency wallets', () => {
    for (const accountType of ['card', 'cash'] as const) {
      expect(walletCountError(accountType, 1)).toBeUndefined()
      expect(walletCountError(accountType, 2)).toBeUndefined()
      expect(canAddCurrencyWallet(accountType)).toBe(true)
    }
  })

  it('preserves all draft wallets when switching account types', () => {
    const wallets = [{ currency: 'CLP' }, { currency: 'USD' }]
    const draft = { account_type: 'card' as const, wallets }

    setDraftAccountType(draft, 'bank')

    expect(draft.account_type).toBe('bank')
    expect(draft.wallets).toBe(wallets)
    expect(draft.wallets.map((wallet) => wallet.currency)).toEqual(['CLP', 'USD'])
    expect(walletCountError(draft.account_type, draft.wallets.length))
      .toBe('multiple_currencies_unsupported_for_account_type')

    setDraftAccountType(draft, 'cash')
    expect(walletCountError(draft.account_type, draft.wallets.length)).toBeUndefined()
  })

  it('hides the add-currency control for the default bank draft', async () => {
    const markup = await renderToString(accountApp())

    expect(markup).toContain('Esta cuenta usa una moneda.')
    expect(markup).not.toContain('Añadir otra moneda')
  })

  it('renders the account form with associated labels and date-only opening control', async () => {
    const markup = await renderToString(accountApp())

    expect(markup).toMatch(/<label for="account-name"[^>]*>Nombre<\/label>/)
    expect(markup).toContain('id="account-name"')
    expect(markup).toMatch(/<label for="wallet-amount-0"[^>]*>Saldo inicial<\/label>/)
    expect(markup).toMatch(/id="wallet-date-0"[^>]*type="date"/)
    expect(markup).toContain('id="account-type"')
    expect(markup).toContain('id="base-currency"')
  })
})
