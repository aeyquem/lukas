import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'
const b=await browser()
try {
  await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
  await b.set('#account-type','investment')
  assert.equal(await b.evaluate(`document.querySelector('.eligibility-field input').checked`),false)
  await b.set('#account-type','card')
  await b.evaluate(`document.querySelector('.eligibility-field input').click()`)
  const name=`Browser liability card ${Date.now()}`
  await b.set('#account-name',name)
  await b.set('#wallet-currency-0','USD')
  await b.set('#wallet-amount-0','-20')
  await b.waitFor(`Boolean(document.querySelector('#wallet-basis-0'))`)
  await b.set('#wallet-basis-0','-20000')
  await b.click('form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('.success-notice'))`)
  await b.evaluate(`Array.from(document.querySelectorAll('.account-card')).find(e=>e.innerText.includes(${JSON.stringify(name)})).click()`)
  await b.waitFor(`Boolean(document.querySelector('#edit-account-name'))`)
  assert.equal(await b.evaluate(`Boolean(document.querySelector('#account-type'))`),false)
  await b.click('.opening-row button')
  await b.set('#correction-amount','-22')
  await b.set('#correction-basis','-22000')
  await b.set('#correction-reason','Browser check of paired foreign debt correction')
  await b.click('.correction-panel button[type="submit"]')
  await b.waitFor(`document.querySelector('.success-notice')?.innerText.includes('Corrección guardada')`)
  const account=await b.evaluate(`fetch('/api/accounts').then(r=>r.json()).then(a=>a.find(x=>x.name===${JSON.stringify(name)}))`)
  const opening=account.wallets[0].openings[0]
  assert.equal(opening.amount,'-22')
  assert.equal(opening.liability_clp_basis,'-22000')
  assert.equal(opening.liability_basis_revision,2)
  console.log(JSON.stringify({result:'PASS',checks:['investment defaults tracking only','type immutable after create','foreign liability paired correction'],account}))
}finally{await b.close()}
