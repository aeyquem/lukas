import { createSSRApp } from 'vue'
import { renderToString } from 'vue/server-renderer'
import { describe, expect, it } from 'vitest'
import AccountCard from '../src/components/accounts/AccountCard.vue'

describe('AccountCard', () => {
  it('shows the authoritative account identity, eligibility, and exact currency balances', async () => {
    const markup = await renderToString(
      createSSRApp(AccountCard, {
        selected: true,
        account: {
          id: 'account-1',
          name: 'Cuenta diaria',
          account_type: 'card',
          base_currency: 'CLP',
          budget_eligible: true,
          wallets: [
            { id: 'wallet-clp', currency: 'CLP', balance: '-1250' },
            { id: 'wallet-usd', currency: 'USD', balance: '1234567.89' },
          ],
        },
      }),
    )

    expect(markup).toContain('Cuenta diaria')
    expect(markup).toContain('Participa en presupuesto')
    expect(markup).toContain('CLP')
    expect(markup).toContain('CLP-1.250')
    expect(markup).toContain('1.234.567,89')
    expect(markup).toContain('aria-current="true"')
  })
})
