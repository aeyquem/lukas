import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

const mode = process.argv[2]
assert.ok(['outage', 'recovered'].includes(mode), 'Use outage or recovered')
const api = process.env.LUKAS_BUDGET_API ?? 'http://localhost:4175'
const baseline = JSON.parse(await readFile('/tmp/lukas-fx-budget-baseline.json', 'utf8'))
const response = await fetch(`${api}/api/budgets/${baseline.period}`)
const budget = await response.json()
assert.equal(response.status, 200, JSON.stringify(budget))
assert.equal(budget.complete, true, 'Stored FX observation should keep the period complete')
assert.equal(budget.funding.state, 'complete')
assert.equal(budget.ready_to_assign.state, 'complete')
assert.equal(budget.funding.amount, baseline.funding.amount)
assert.equal(budget.ready_to_assign.amount, baseline.readyToAssign.amount)

console.log(JSON.stringify({
  result: 'PASS',
  mode,
  period: budget.period,
  funding: budget.funding,
  readyToAssign: budget.ready_to_assign,
}))
