import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'

const b = await browser()
try {
  const seed = await b.evaluate(`(async()=>{
    const today=new Intl.DateTimeFormat('en-CA',{timeZone:'America/Santiago',year:'numeric',month:'2-digit',day:'2-digit'}).format(new Date())
    const generation=(await (await fetch('/api/bootstrap')).json()).installation_generation
    async function create(name,type,wallets,budgetEligible=false){
      const response=await fetch('/api/accounts',{method:'POST',headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':generation},body:JSON.stringify({name,account_type:type,base_currency:'CLP',budget_eligible:budgetEligible,wallets})})
      const value=await response.json();if(!response.ok)throw new Error(JSON.stringify(value));return fetch('/api/accounts/'+value.resource_id).then(r=>r.json())
    }
    const bank=await create('Browser transaction bank '+Date.now(),'bank',[{currency:'CLP',opening_amount:'50000',opening_date:'2020-01-01'}])
    const cash=await create('Browser transaction cash '+Date.now(),'cash',[{currency:'CLP',opening_amount:'0',opening_date:'2020-01-01'},{currency:'USD',opening_amount:'1000',opening_date:'2020-01-01'}])
    const destination=await create('Browser transaction reserve '+Date.now(),'cash',[{currency:'CLP',opening_amount:'0',opening_date:'2020-01-01'}])
    const opener=await create('Browser same-day opener '+Date.now(),'savings',[{currency:'CLP',opening_amount:'2000',opening_date:today}])
    return {today,bank,cash,destination,opener}
  })()`)

  await b.command('Page.navigate', { url: `${b.origin}/transactions` })
  await b.waitFor(`Boolean(document.querySelector('#transaction-wallet') && document.querySelector('#transaction-wallet').options.length>1)`)
  const findTransactionByNote = (note) => b.evaluate(`(async()=>{let cursor=null,items=[];do{const q=new URLSearchParams({page_size:'100'});if(cursor){q.set('before_date',cursor.before_date);q.set('before_id',cursor.before_id)}const page=await (await fetch('/api/transactions?'+q)).json();items.push(...page.items);cursor=page.next}while(cursor);return items.find(item=>item.note===${JSON.stringify(note)})||null})()`)
  const bankWallet = seed.bank.wallets[0].id
  const cashClp = seed.cash.wallets.find((wallet) => wallet.currency === 'CLP').id
  const cashUsd = seed.cash.wallets.find((wallet) => wallet.currency === 'USD').id
  const reserveClp = seed.destination.wallets[0].id
  const openingWallet = seed.opener.wallets[0].id
  const category = await b.evaluate(`document.querySelector('#transaction-category').options[1].value`)
  const suffix = Date.now()
  const submit = async () => {
    await b.click('#transaction-editor form button[type="submit"]')
    await b.waitFor(`Boolean(document.querySelector('.success-notice')?.innerText.includes('guardado'))`)
  }
  const single = async (kind, amount, wallet = bankWallet, note = `Browser transaction ${suffix}`) => {
    await b.set('#transaction-kind', kind)
    await b.set('#transaction-wallet', wallet)
    await b.set('#transaction-amount', amount)
    await b.set('#transaction-date', seed.today)
    await b.set('#transaction-note', note)
  }

  const incomeNote = `Browser income ${suffix}`
  await single('income', '1000', bankWallet, incomeNote)
  await submit()

  const expenseNote = `Browser expense ${suffix}`
  await single('expense', '100', bankWallet, expenseNote)
  await b.set('#transaction-category', category)
  await submit()
  const expense = await findTransactionByNote(expenseNote)
  assert.ok(expense)

  const refundNote = `Browser linked refund ${suffix}`
  await single('refund', '20', bankWallet, refundNote)
  while (!await b.evaluate(`Array.from(document.querySelector('#refund-source').options).some(e=>e.value===${JSON.stringify(expense.id)})`)) {
    await b.click('.pagination-actions button')
    await b.waitFor(`!document.querySelector('.pagination-actions button')?.disabled`)
  }
  await b.set('#refund-source', expense.id)
  await submit()

  const sameTransferNote = `Browser same-currency transfer ${suffix}`
  await b.set('#transaction-kind', 'transfer')
  await b.set('#transaction-date', seed.today)
  await b.set('#source-wallet', bankWallet)
  await b.set('#destination-wallet', cashClp)
  await b.set('#source-amount', '100')
  await b.set('#destination-amount', '100')
  await b.set('#transaction-note', sameTransferNote)
  await submit()

  const crossTransferNote = `Browser cross-currency transfer ${suffix}`
  await b.set('#source-wallet', cashUsd)
  await b.set('#destination-wallet', reserveClp)
  await b.set('#source-amount', '10')
  await b.set('#destination-amount', '9300')
  await b.set('#transaction-note', crossTransferNote)
  await b.click('#transaction-editor form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('#transaction-conversion-evidence-error'))`)
  assert.equal(await findTransactionByNote(crossTransferNote), null)
  await b.set('#conversion-evidence', '{"source":"browser statement","rate":"930"}')
  await submit()

  // Empty date and nonpositive money are rejected client-side and never sent.
  const invalidNote = `Invalid transaction ${suffix}`
  await single('expense', '0', bankWallet, invalidNote)
  await b.click('#transaction-editor form button[type="submit"]')
  await b.waitFor(`document.querySelector('#transaction-amount-error')?.textContent.includes('positivo')`)
  await b.set('#transaction-amount', '5')
  await b.set('#transaction-date', '')
  await b.click('#transaction-editor form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('#transaction-business-date-error'))`)
  assert.equal(await findTransactionByNote(invalidNote), null)

  // Same-day opening chronology requires a per-wallet choice before the write.
  await b.set('#transaction-date', seed.today)
  await b.set('#transaction-wallet', openingWallet)
  await b.set('#transaction-amount', '2')
  await b.waitFor(`Boolean(document.querySelector('#wallet-opening-treatment-apply'))`)
  await b.click('#transaction-editor form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('#wallet-opening-treatment-error'))`)
  assert.equal(await b.evaluate(`document.querySelector('#wallet-opening-treatment-apply').getAttribute('aria-invalid')`), 'true')
  await b.click('#wallet-opening-treatment-apply')
  await b.set('#transaction-note', `Browser opening-date income ${suffix}`)
  await submit()

  // Correct the categorized expense: a reason is required, then the revision advances once.
  while (!await b.evaluate(`Array.from(document.querySelectorAll('.activity-list tbody tr')).some(e=>e.innerText.includes(${JSON.stringify(expenseNote)}))`)) {
    await b.click('.pagination-actions button')
    await b.waitFor(`!document.querySelector('.pagination-actions button')?.disabled`)
  }
  await b.evaluate(`(()=>{const article=Array.from(document.querySelectorAll('.activity-list tbody tr')).find(e=>e.innerText.includes(${JSON.stringify(expenseNote)}));article.querySelector('button[aria-label^="Editar"]').click()})()`)
  await b.waitFor(`Boolean(document.querySelector('#correction-reason'))`)
  await b.set('#transaction-amount', '110')
  await b.click('#transaction-editor form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('#transaction-reason-error'))`)
  await b.set('#correction-reason', 'Browser correction flow verification')
  await b.click('#transaction-editor form button[type="submit"]')
  await b.waitFor(`fetch('/api/transactions/${expense.id}').then(r=>r.json()).then(t=>t.revision===2)`)
  const corrected = await b.evaluate(`fetch('/api/transactions/${expense.id}').then(r=>r.json())`)
  assert.equal(corrected.effects[0].amount, '-110')

  const linked = await findTransactionByNote(refundNote)
  assert.equal(linked.source_expense_id, expense.id)
  assert.equal(linked.category_id, category)

  const unvaluedForeignNote = `Browser unvalued USD expense ${suffix}`
  await single('expense', '12.50', cashUsd, unvaluedForeignNote)
  await b.set('#transaction-category', category)
  await submit()
  const unvaluedForeign = await findTransactionByNote(unvaluedForeignNote)
  assert.equal(unvaluedForeign.effects[0].currency, 'USD')
  assert.equal(unvaluedForeign.effects[0].amount, '-12.50')
  assert.equal(unvaluedForeign.confirmed_clp_amount, null)

  const valuedForeignNote = `Browser confirmed USD expense ${suffix}`
  await single('expense', '13', cashUsd, valuedForeignNote)
  await b.set('#transaction-category', category)
  await b.set('#confirmed-clp', '12090')
  await submit()
  const valuedForeign = await findTransactionByNote(valuedForeignNote)
  assert.equal(valuedForeign.effects[0].currency, 'USD')
  assert.equal(valuedForeign.confirmed_clp_amount, '12090')

  const cross = await findTransactionByNote(crossTransferNote)
  assert.equal(cross.kind, 'transfer')
  assert.deepEqual(cross.conversion_evidence, { source: 'browser statement', rate: '930' })

  // Seed 250 tracking-only transfers, then compare the rendered pages with the deterministic API order.
  const pageSeed = await b.evaluate(`(async()=>{
    const generation=(await (await fetch('/api/bootstrap')).json()).installation_generation
    const today=${JSON.stringify(seed.today)},from=${JSON.stringify(cashClp)},to=${JSON.stringify(reserveClp)},tag=${JSON.stringify(`Browser pagination ${suffix}`)},ids=[]
    for(let i=0;i<250;i++){
      const response=await fetch('/api/transactions',{method:'POST',headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':generation},body:JSON.stringify({kind:'transfer',business_date:today,source_wallet_id:from,source_amount:'1',destination_wallet_id:to,destination_amount:'1',note:tag+' '+String(i).padStart(3,'0')})})
      const receipt=await response.json();if(!response.ok)throw new Error(JSON.stringify(receipt));ids.push(receipt.resource_id)
    }
    async function all(){let cursor=null,items=[];do{const q=new URLSearchParams({page_size:'100'});if(cursor){q.set('before_date',cursor.before_date);q.set('before_id',cursor.before_id)}const p=await (await fetch('/api/transactions?'+q)).json();items.push(...p.items);cursor=p.next}while(cursor);return items.map(t=>t.note || '—')}
    return {ids,all:await all(),tag}
  })()`)
  assert.equal(pageSeed.ids.length, 250)
  await b.evaluate('window.__transactionBeforeReload = true')
  await b.command('Page.reload')
  await b.waitFor(`window.__transactionBeforeReload !== true && document.querySelectorAll('.activity-list tbody tr').length===50`)
  while (await b.evaluate(`Boolean(document.querySelector('.pagination-actions button'))`)) {
    const previousCount = await b.evaluate(`document.querySelectorAll('.activity-list tbody tr').length`)
    await b.click('.pagination-actions button')
    await b.waitFor(`document.querySelectorAll('.activity-list tbody tr').length>${previousCount}`)
  }
  const renderedIds = await b.evaluate(`Array.from(document.querySelectorAll('.activity-list tbody tr')).map(e=>e.querySelector('.note-cell').textContent)`)
  assert.deepEqual(renderedIds, pageSeed.all)
  assert.equal(renderedIds.filter(note => note.startsWith(pageSeed.tag)).length, 250)
  assert.equal(new Set(renderedIds.filter(note => note.startsWith(pageSeed.tag))).size, 250)

  console.log(JSON.stringify({
    result: 'PASS',
    checks: [
      'income and categorized expense', 'linked refund follows source category',
      'unvalued foreign expense is retained in source currency', 'confirmed foreign statement value is retained',
      'same- and cross-currency transfers retain evidence', 'date and positive-amount validation prevents writes',
      'same-day opening prompt is required and accessible', 'correction requires reason and advances revision',
      '250-record deterministic pagination has no gaps or duplicates',
    ],
    accountIds: [seed.bank.id, seed.cash.id, seed.destination.id, seed.opener.id],
    transactionIds: [expense.id, linked.id, cross.id, corrected.id],
    paginationRecords: renderedIds.length,
  }))
} finally { await b.close() }
