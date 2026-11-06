import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'

const b = await browser()
try {
  const account = await b.evaluate(`(async()=>{
    const accounts=await (await fetch('/api/accounts')).json()
    let row=accounts.find(item=>item.name.startsWith('Browser transaction bank '))
    if(!row){
      const generation=(await (await fetch('/api/bootstrap')).json()).installation_generation
      const response=await fetch('/api/accounts',{method:'POST',headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':generation},body:JSON.stringify({name:'Browser retry account '+Date.now(),account_type:'bank',base_currency:'CLP',budget_eligible:false,wallets:[{currency:'CLP',opening_amount:'1000',opening_date:'2020-01-01'}]})})
      const receipt=await response.json();if(!response.ok)throw new Error(JSON.stringify(receipt));row=await (await fetch('/api/accounts/'+receipt.resource_id)).json()
    }
    return row
  })()`)
  assert.ok(account.wallets[0]?.openings[0], 'the retry test needs an existing dated opening')
  const opening = account.wallets[0].openings[0]

  await b.command('Page.reload')
  await b.waitFor('document.readyState === \'complete\'')
  await b.waitFor(`Boolean(document.querySelector('.account-card'))`)
  await b.evaluate(`Array.from(document.querySelectorAll('.account-card')).find(e=>e.innerText.includes(${JSON.stringify(account.name)})).click()`)
  await b.waitFor(`Boolean(document.querySelector('.opening-row button'))`)
  await b.evaluate(`document.querySelector('.opening-row button').click()`)
  await b.waitFor(`Boolean(document.querySelector('#correction-amount'))`)

  // Persist-before-send is checked in the fetch interceptor, before it calls the real API.
  await b.evaluate(`(()=>{const real=window.fetch;window.fetch=async(...args)=>{
    const [url,options]=args
    if(String(url).includes('/openings/')&&options?.method==='PUT'&&!window.firstWrite){
      const headers=Object.fromEntries(new Headers(options.headers))
      const stored=Object.keys(localStorage).filter(k=>k.startsWith('lukas.pending-financial-write.v1:')).map(k=>JSON.parse(localStorage.getItem(k))).find(command=>command.route===String(url)&&command.idempotencyKey===headers['idempotency-key'])
      window.persistedBeforeFirstFinancialSend=stored||null
      const response=await real(...args)
      window.firstReceipt=await response.clone().json()
      window.firstWrite={route:String(url),method:options.method,body:options.body,headers}
      throw new TypeError('Simulated lost commit response')
    }
    return real(...args)
  }})()`)

  const correctedAmount = (BigInt(opening.amount.split('.')[0]) + 1n).toString()
  await b.set('#correction-amount', correctedAmount)
  await b.set('#correction-reason', 'Browser verification of exact retry after a lost response')
  await b.click('.correction-panel button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('.pending-commands'))`)

  const first = await b.evaluate('window.firstWrite')
  const persisted = await b.evaluate('window.persistedBeforeFirstFinancialSend')
  const firstReceipt = await b.evaluate('window.firstReceipt')
  assert.ok(persisted, 'pending bytes must exist before the first financial fetch')
  assert.equal(persisted.route, first.route)
  assert.equal(persisted.method, first.method)
  assert.equal(persisted.body, first.body)
  assert.equal(persisted.idempotencyKey, first.headers['idempotency-key'])
  assert.equal(persisted.installationGeneration, first.headers['installation-generation'])

  const storedBeforeCancel = await b.evaluate(`Object.keys(localStorage).filter(k=>k.startsWith('lukas.pending-financial-write.v1:')).map(k=>localStorage.getItem(k))`)
  // Cancel closes the correction editor but must not remove the unresolved request.
  await b.click('.correction-panel button.secondary-button')
  assert.deepEqual(await b.evaluate(`Object.keys(localStorage).filter(k=>k.startsWith('lukas.pending-financial-write.v1:')).map(k=>localStorage.getItem(k))`), storedBeforeCancel)
  await b.evaluate(`Array.from(document.querySelectorAll('.pending-commands button')).find(e=>e.innerText==='Ocultar aviso').click()`)
  assert.deepEqual(await b.evaluate(`Object.keys(localStorage).filter(k=>k.startsWith('lukas.pending-financial-write.v1:')).map(k=>localStorage.getItem(k))`), storedBeforeCancel)
  await b.evaluate('window.__beforeReload = true')
  await b.command('Page.reload')
  await b.waitFor('window.__beforeReload !== true && document.readyState === \'complete\'')
  await b.waitFor(`Boolean(document.querySelector('.pending-commands'))`)

  await b.evaluate(`(()=>{const real=window.fetch;window.fetch=async(...args)=>{
    const [url,options]=args
    if(String(url).includes('/openings/')&&options?.method==='PUT'){
      const response=await real(...args)
      window.retryReceipt=await response.clone().json()
      window.retryWrite={route:String(url),method:options.method,body:options.body,headers:Object.fromEntries(new Headers(options.headers))}
      return response
    }
    return real(...args)
  }})()`)
  await b.evaluate(`Array.from(document.querySelectorAll('.pending-commands button')).find(e=>e.innerText==='Reintentar').click()`)
  await b.waitFor(`!document.querySelector('.pending-commands')`)

  const retry = await b.evaluate('window.retryWrite')
  const retryReceipt = await b.evaluate('window.retryReceipt')
  assert.deepEqual(retry, first, 'retry must preserve route, method, body, and every request header')
  assert.deepEqual(retryReceipt, firstReceipt, 'exact retry must return the original saved receipt')
  const readback = await b.evaluate(`fetch('/api/accounts/${account.id}').then(r=>r.json())`)
  assert.equal(readback.wallets.find((wallet) => wallet.id === account.wallets[0].id).openings.find((item) => item.id === opening.id).amount, correctedAmount)
  assert.equal(readback.wallets[0].openings.length, account.wallets[0].openings.length)

  console.log(JSON.stringify({
    result: 'PASS',
    checks: [
      'pending command persisted before first financial fetch', 'cancel retains unresolved request',
      'dismiss retains bytes', 'reload rediscovers pending request',
      'retry preserves method/route/body/key/generation', 'retry receipt matches lost response receipt',
      'one corrected opening is present after exact retry',
    ],
    accountId: account.id,
    openingId: opening.id,
    idempotencyKey: first.headers['idempotency-key'],
    receipt: retryReceipt,
  }))
} finally { await b.close() }
