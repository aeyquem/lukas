import assert from 'node:assert/strict'
import { writeFile } from 'node:fs/promises'

const api = process.env.LUKAS_BUDGET_API ?? 'http://localhost:4175'
const current = process.env.LUKAS_BUDGET_CURRENT ?? '2026-11'
const name = `Browser FX outage asset ${current}`
const origin = process.env.LUKAS_BROWSER_ORIGIN ?? 'http://localhost:4174'
const read = async (path) => {
  const response = await fetch(`${api}${path}`)
  const body = await response.json()
  assert.equal(response.status, 200, JSON.stringify(body))
  return body
}

const bootstrap = await read('/api/bootstrap')
const accounts = await read('/api/accounts')
let account = accounts.find((candidate) => candidate.name === name)
if (!account) {
  const response = await fetch(`${api}/api/accounts`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Origin: origin,
      'Idempotency-Key': crypto.randomUUID(),
      'Installation-Generation': bootstrap.installation_generation,
    },
    body: JSON.stringify({
      name,
      account_type: 'cash',
      base_currency: 'CLP',
      budget_eligible: true,
      wallets: [{ currency: 'USD', opening_amount: '100', opening_date: `${current}-01` }],
    }),
  })
  const receipt = await response.json()
  assert.equal(response.status, 201, JSON.stringify(receipt))
  account = await read(`/api/accounts/${receipt.resource_id}`)
}
const wallet = account.wallets.find((candidate) => candidate.currency === 'USD')
assert.ok(wallet)
assert.equal(wallet.balance, '100')

const budget = await read(`/api/budgets/${current}`)
const baseline = {
  result: 'SEEDED',
  accountId: account.id,
  walletId: wallet.id,
  period: budget.period,
  complete: budget.complete,
  funding: budget.funding,
  readyToAssign: budget.ready_to_assign,
}
assert.equal(baseline.complete, true, 'A fresh provider observation should value the eligible USD asset')
assert.equal(baseline.funding.state, 'complete')
await writeFile('/tmp/lukas-fx-budget-baseline.json', JSON.stringify(baseline))
console.log(JSON.stringify(baseline))
