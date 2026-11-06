import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'
const b = await browser()
try {
  assert.equal(b.origin, 'http://localhost:4174', 'Use isolated verification database')
  const name = `Account draft conflict ${Date.now()}`
  await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
  await b.set('#account-name', name)
  await b.click('form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('.success-notice'))`)
  await b.evaluate(`Array.from(document.querySelectorAll('.account-card')).find(e=>e.innerText.includes(${JSON.stringify(name)})).click()`)
  await b.waitFor(`Boolean(document.querySelector('#edit-account-name'))`)
  await b.set('#edit-account-name', `${name} local dirty`)
  // A separate writer advances the revision while this form retains its baseline.
  await b.evaluate(`(async()=>{
    const account=(await (await fetch('/api/accounts')).json()).find(a=>a.name===${JSON.stringify(name)});
    const bootstrap=await (await fetch('/api/bootstrap')).json();
    const response=await fetch('/api/accounts/'+account.id,{method:'PUT',headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':bootstrap.installation_generation},body:JSON.stringify({name:${JSON.stringify(name + ' server')},expected_revision:account.revision})});
    if(!response.ok)throw new Error('Concurrent rename '+response.status);
  })()`)
  await b.evaluate(`Array.from(document.querySelectorAll('button')).find(e=>e.innerText.trim()==='Actualizar cuentas').click()`)
  await b.waitFor(`document.querySelector('#edit-title')?.innerText===${JSON.stringify(name + ' server')}`)
  assert.equal(await b.evaluate(`document.querySelector('#edit-account-name').value`), `${name} local dirty`)
  await b.click('.name-edit button[type="submit"]')
  await b.waitFor(`document.querySelector('.error-notice')?.innerText.includes('cambió')`)
  assert.equal(await b.evaluate(`document.querySelector('#edit-account-name').value`), `${name} local dirty`)
  await b.evaluate(`Array.from(document.querySelectorAll('button')).find(e=>e.innerText.trim()==='Revisar y recargar nombre').click()`)
  await b.waitFor(`document.querySelector('#edit-account-name')?.value===${JSON.stringify(name + ' server')}`)
  await b.click('.opening-row button')
  await b.set('#correction-amount', '123')
  await b.set('#correction-reason', 'Keep my correction draft')
  await b.evaluate(`(async()=>{
    const account=(await (await fetch('/api/accounts')).json()).find(a=>a.name===${JSON.stringify(name + ' server')});
    const opening=account.wallets[0].openings[0];
    const bootstrap=await (await fetch('/api/bootstrap')).json();
    const response=await fetch('/api/accounts/'+account.id+'/openings/'+opening.id,{method:'PUT',headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':bootstrap.installation_generation},body:JSON.stringify({amount:'1000',expected_revision:opening.revision,reason:'Concurrent authoritative correction'})});
    if(!response.ok)throw new Error('Concurrent correction '+response.status);
  })()`)
  await b.evaluate(`Array.from(document.querySelectorAll('button')).find(e=>e.innerText.trim()==='Actualizar cuentas').click()`)
  await b.waitFor(`document.querySelector('.wallet-readback')?.innerText.includes('1.000')`)
  await b.click('.correction-panel button[type="submit"]')
  await b.waitFor(`document.querySelector('.correction-panel .error-notice')?.innerText.includes('cambió')`)
  assert.equal(await b.evaluate(`document.querySelector('#correction-amount').value`), '123')
  assert.equal(await b.evaluate(`document.querySelector('#correction-reason').value`), 'Keep my correction draft')
  console.log(JSON.stringify({result:'PASS',checks:['dirty name survives authoritative refetch','captured revision returns 409','conflict retains draft','explicit reload adopts current name and revision','correction baseline remains stale after refetch','correction 409 preserves amount and reason']}))
} finally { await b.close() }
