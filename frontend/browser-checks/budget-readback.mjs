import assert from 'node:assert/strict'
const current=process.env.LUKAS_BUDGET_CURRENT
assert.match(current??'',/^\d{4}-(0[1-9]|1[0-2])$/)
const [year,month]=current.split('-').map(Number)
const previous=month===1?`${year-1}-12`:`${year}-${String(month-1).padStart(2,'0')}`
const base=process.env.LUKAS_BUDGET_API??'http://localhost:4175'
const read=async path=>{const r=await fetch(base+path);const body=await r.json();assert.equal(r.status,200,JSON.stringify(body));return body}
const closed=await read(`/api/budgets/${previous}`),next=await read(`/api/budgets/${current}`)
const category=closed.categories.find(c=>c.name.startsWith('Isolated browser category '))
assert.equal(closed.closed,true)
assert.equal(closed.complete,true)
assert.equal(category.assigned,'700')
assert.equal(category.assignment_intent,'750')
assert.equal(category.activity.amount,'100')
assert.equal(next.ready_to_assign.amount,'650')
const carry=closed.categories.find(c=>c.id!==category.id)
assert.equal(carry.assigned,'200')
assert.equal(next.categories.find(c=>c.id===carry.id).remaining.amount,'200')
console.log(JSON.stringify({result:'PASS',previous,current,checks:['release/carry close snapshot','closed assignment preserves700 and records750 intent','confirmed expense activity100','next period ready-to-assign650','carried category remainder200'],categoryId:category.id}))
