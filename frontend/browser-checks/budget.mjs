import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'
const origin = process.env.LUKAS_BROWSER_ORIGIN
assert.equal(origin, 'http://localhost:4174', 'Budget close checks require the isolated browser origin')
const b = await browser()
const resumePhase = process.env.LUKAS_BUDGET_RESUME
assert.ok(resumePhase === undefined || ['archive','close'].includes(resumePhase), 'Supported debug resume phases are archive and close')
const resume = resumePhase !== undefined
const checks = []
const json = value => JSON.stringify(value)
try {
  await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
  const accounts = await b.evaluate(`fetch('/api/accounts').then(r=>r.json())`)
  if (resume) {
    const fixtureNames = ['Isolated CLP bank','Isolated foreign card']
    assert.deepEqual(accounts.filter(a=>fixtureNames.includes(a.name)).map(a=>a.name).sort(), fixtureNames, 'Resume requires the two isolated fixture accounts')
    for (const extra of accounts.filter(a=>!fixtureNames.includes(a.name))) {
      assert.match(extra.name,/^Browser (firefox|zen|zen-browser) /,'Resume rejects unrelated account data')
      assert.equal(extra.account_type,'bank')
      for (const wallet of extra.wallets) {
        assert.match(wallet.balance,/^-?0(?:\.0+)?$/,'Compatibility fixture must have zero balance')
        for (const opening of wallet.openings) assert.match(opening.amount,/^-?0(?:\.0+)?$/,'Compatibility fixture must have zero openings')
      }
    }
  } else assert.deepEqual(accounts, [], 'Use a fresh isolated database; this script closes its first period')
  await b.evaluate(`window.budgetRead = path => fetch(path).then(async r=>{const body=await r.json();if(!r.ok)throw new Error(JSON.stringify(body));return body});window.budgetWrite = async(method,path,body)=>{const g=await window.budgetRead('/api/bootstrap');const r=await fetch(path,{method,headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':g.installation_generation},body:JSON.stringify(body)});const result=await r.json();if(!r.ok)throw new Error(JSON.stringify({status:r.status,result}));return result}`)
  const read = path => b.evaluate(`window.budgetRead(${json(path)})`)
  const write = (method,path,body) => b.evaluate(`window.budgetWrite(${json(method)},${json(path)},${json(body)})`)
  const current = await b.evaluate(`new Intl.DateTimeFormat('en-CA',{timeZone:'America/Santiago',year:'numeric',month:'2-digit'}).format(new Date())`)
  const [year,month] = current.split('-').map(Number)
  const previous = month===1 ? `${year-1}-12` : `${year}-${String(month-1).padStart(2,'0')}`
  if (!resume) for (const [name,type,currency,amount] of [['Isolated CLP bank','bank','CLP','1000'],['Isolated foreign card','card','USD','0']]) {
    await write('POST','/api/accounts',{name,account_type:type,base_currency:'CLP',budget_eligible:true,wallets:[{currency,opening_amount:amount,opening_date:`${previous}-01`}]})
  }
  const seeded = await read('/api/accounts')
  const foreignWallet = seeded.find(a=>a.name==='Isolated foreign card').wallets[0].id
  await b.command('Page.navigate',{url:`${origin}/budget`})
  await b.waitFor(`Boolean(document.querySelector('.category-new-form input'))`)
  // A navigation starts a new JS context; restore the API helpers.
  await b.evaluate(`window.budgetRead = path => fetch(path).then(async r=>{const body=await r.json();if(!r.ok)throw new Error(JSON.stringify(body));return body});window.budgetWrite = async(method,path,body)=>{const g=await window.budgetRead('/api/bootstrap');const r=await fetch(path,{method,headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':g.installation_generation},body:JSON.stringify(body)});const result=await r.json();if(!r.ok)throw new Error(JSON.stringify({status:r.status,result}));return result}`)
  await b.evaluate(`window.budgetTrace=[];(()=>{const real=window.fetch;window.fetch=async(...args)=>{const [url,options]=args;const response=await real(...args);if(String(url).includes('/api/budgets/')){const body=await response.clone().json();window.budgetTrace.push({url:String(url),method:options?.method??'GET',status:response.status,request:options?.body??null,responseRevision:body.revision??null,responseError:body.error??null,uiRequest:new Headers(options?.headers).has('Accept')})}return response}})()`)
  const name = `Isolated browser category ${Date.now()}`
  let category
  if (resume) category = (await read('/api/categories')).find(c=>c.name.startsWith('Isolated browser category '))
  else {
    await b.set('.category-new-form input:not([type=number])',name)
    await b.set('.category-new-form input[type=number]','1000')
    await b.click('.category-new-form button')
    await b.waitFor(`fetch('/api/categories').then(r=>r.json()).then(a=>a.some(c=>c.name===${json(name)}))`)
    category = (await read('/api/categories')).find(c=>c.name===name)
  }
  assert.ok(category, 'Isolated category fixture exists')
  const waitIdle = () => b.waitFor(`document.querySelector('.category-new-form button')?.disabled===false && !Array.from(document.querySelectorAll('[role=status]')).some(e=>e.innerText==='Cargando presupuesto…')`)
  const editCategory = async () => {
    await waitIdle()
    await b.evaluate(`Array.from(document.querySelectorAll('.category-item')).find(e=>e.innerText.includes(${json(category.name)})).querySelector('button').click()`)
    await b.waitFor(`Boolean(document.querySelector('#category-name-${category.id}'))`)
  }
  const categoryButton = async text => {
    const expression = `Array.from(document.querySelectorAll('#category-name-${category.id}')).map(e=>e.closest('article')).flatMap(e=>Array.from(e.querySelectorAll('button'))).find(e=>e.innerText===${json(text)})`
    await b.waitFor(`Boolean(${expression}) && !(${expression}).disabled`)
    await b.evaluate(`(async()=>{(${expression}).click();await new Promise(r=>setTimeout(r,0))})()`)
    await waitIdle()
  }
  if (!resume) {
  await editCategory()
  await b.set(`#category-name-${category.id}`,`${name} renamed`)
  await b.set(`#category-order-${category.id}`,'999')
  await categoryButton('Guardar')
  await b.waitFor(`fetch('/api/categories').then(r=>r.json()).then(a=>a.some(c=>c.id===${json(category.id)}&&c.sort_order===999&&c.name===${json(`${name} renamed`)}))`)
  category = (await read('/api/categories')).find(c=>c.id===category.id)
  await editCategory(); await categoryButton('Archivar')
  await b.waitFor(`fetch('/api/categories').then(r=>r.json()).then(a=>a.find(c=>c.id===${json(category.id)}).archived)`)
  await editCategory(); await categoryButton('Restaurar')
  await b.waitFor(`fetch('/api/categories').then(r=>r.json()).then(a=>!a.find(c=>c.id===${json(category.id)}).archived)`)
  checks.push('category create/rename/reorder/archive/restore through UI')
  } else checks.push('resumed isolated fixture after previous category CRUD verification')
  const expense = resume
    ? await b.evaluate(`(async()=>{let cursor=null,items=[];do{const q=new URLSearchParams({page_size:'100'});if(cursor){q.set('before_date',cursor.before_date);q.set('before_id',cursor.before_id)}const page=await(await fetch('/api/transactions?'+q)).json();items.push(...page.items);cursor=page.next}while(cursor);const item=items.find(t=>t.note==='Isolated incomplete foreign expense');return item?{resource_id:item.id}:null})()`)
    : await write('POST','/api/transactions',{kind:'expense',wallet_id:foreignWallet,amount:'10',business_date:`${previous}-02`,category_id:category.id,note:'Isolated incomplete foreign expense'})
  await b.evaluate(`(()=>{const e=document.querySelector('#budget-period');e.value=${json(previous)};e.dispatchEvent(new Event('change',{bubbles:true}))})()`)
  await b.waitFor(`Boolean(document.querySelector('#assign-${category.id}'))`)
  const assignmentSelector = `#assign-${category.id}`
  const submitAssignment = async id => {
    const expression = `document.querySelector('#assign-${id}')?.closest('form').querySelector('button')`
    await waitIdle()
    await b.waitFor(`Boolean(${expression}) && !(${expression}).disabled`)
    await b.evaluate(`(async()=>{(${expression}).click();await new Promise(r=>setTimeout(r,0))})()`)
    await waitIdle()
  }
  if (resumePhase !== 'close') {
  const snapshot = await read(`/api/budgets/${previous}`)
  assert.equal(snapshot.complete,false)
  assert.equal(snapshot.activity.state,'incomplete','Unvalued expense must make total activity incomplete')
  const incompleteCategory = snapshot.categories.find(c=>c.id===category.id)
  assert.equal(incompleteCategory.activity.state,'incomplete','Unvalued expense must make category activity incomplete')
  assert.equal(incompleteCategory.remaining.state,'incomplete','Unvalued expense must make category remaining incomplete')
  assert.ok(incompleteCategory.activity.unresolved.some(v=>v.currency==='USD'&&v.amount==='10'),'Original unresolved USD amount must be exposed')
  // Seeded transactions were written outside the component; explicitly refresh
  // its authoritative view after selecting the period.
  await b.click('.page-heading > button')
  await b.waitFor(`document.body.innerText.includes('USD')`)
  await waitIdle()
  await b.set(assignmentSelector,'500'); await submitAssignment(category.id)
  await b.waitFor(`fetch('/api/budgets/${previous}').then(r=>r.json()).then(p=>p.categories.find(c=>c.id===${json(category.id)}).assigned==='500')`)
  checks.push('incomplete original USD expense is visible and assignment remains savable')
  await editCategory(); await categoryButton('Archivar')
  await b.waitFor(`document.querySelector('.category-manager .notice-error')?.innerText.includes('actividad o asignaciones')`)
  assert.equal((await read('/api/categories')).find(c=>c.id===category.id).archived,false)
  await categoryButton('Cancelar')
  checks.push('archive in-use category rejected without hiding it')
  // Change the server revision after the browser's baseline, then submit its draft.
  await b.set(assignmentSelector,'700')
  const old = await read(`/api/budgets/${previous}`)
  await write('PUT',`/api/budgets/${previous}/assignments/${category.id}`,{amount:'600',expected_period_revision:old.revision,expected_revision:old.categories.find(c=>c.id===category.id).assignment_revision})
  await submitAssignment(category.id)
  await b.waitFor(`document.body.innerText.includes('borrador')`)
  assert.equal(await b.evaluate(`document.querySelector(${json(assignmentSelector)}).value`),'700')
  await submitAssignment(category.id)
  await b.waitFor(`fetch('/api/budgets/${previous}').then(r=>r.json()).then(p=>p.categories.find(c=>c.id===${json(category.id)}).assigned==='700')`)
  checks.push('stale assignment retains local draft and subsequent save refetches authoritative value')
  await b.click('.close-panel > button')
  await b.waitFor(`Boolean(document.querySelector('.close-preview button'))`)
  assert.equal(await b.evaluate(`document.querySelector('.close-preview button').disabled`),true)
  assert.match(await b.evaluate(`document.querySelector('.close-preview').innerText`),/incomplete_values/)
  await write('PUT',`/api/transactions/${expense.resource_id}`,{kind:'expense',wallet_id:foreignWallet,amount:'10',business_date:`${previous}-02`,category_id:category.id,confirmed_clp_amount:'100',expected_revision:1,reason:'Confirm statement CLP amount'})
  await b.click('.page-heading > button')
  await b.waitFor(`!document.body.innerText.includes('subtotal conocido')`)
  await b.set(assignmentSelector,'1200'); await submitAssignment(category.id)
  await b.waitFor(`fetch('/api/budgets/${previous}').then(r=>r.json()).then(p=>p.categories.find(c=>c.id===${json(category.id)}).assigned==='1200')`)
  assert.equal((await read(`/api/budgets/${previous}`)).ready_to_assign.amount,'-200')
  await b.set(assignmentSelector,'700'); await submitAssignment(category.id)
  await b.waitFor(`fetch('/api/budgets/${previous}').then(r=>r.json()).then(p=>p.categories.find(c=>c.id===${json(category.id)}).assigned==='700')`)
  checks.push('over-assignment preserves a signed negative ready-to-assign value')
  } else {
    const confirmed = await read(`/api/budgets/${previous}`)
    assert.equal(confirmed.closed,false,'Close resume requires an unclosed isolated period')
    assert.equal(confirmed.complete,true,'Close resume requires the confirmed expense phase')
    assert.equal(confirmed.categories.find(c=>c.id===category.id).assigned,'700')
    await b.click('.page-heading > button')
    await waitIdle()
    checks.push('resumed completed incomplete/stale/over-assignment phases before period close')
  }
  const carryCategory = (await read(`/api/budgets/${previous}`)).categories.find(c=>c.id!==category.id)
  await b.set(`#assign-${carryCategory.id}`,'200'); await submitAssignment(carryCategory.id)
  await b.waitFor(`fetch('/api/budgets/${previous}').then(r=>r.json()).then(p=>p.categories.find(c=>c.id===${json(carryCategory.id)}).assigned==='200')`)
  await b.click('.close-panel > button')
  await b.waitFor(`document.querySelector('.close-preview button')?.disabled===false`)
  await b.click(`input[name="disposition-${category.id}"][value="release"]`)
  await b.click(`input[name="disposition-${carryCategory.id}"][value="carry"]`)
  const beforeClose = await read(`/api/budgets/${previous}`)
  assert.equal(beforeClose.ready_to_assign.amount,'100')
  await b.click('.close-preview button')
  await b.waitFor(`document.querySelector('#budget-period')?.value===${json(current)} && document.body.innerText.includes('El período se cerró')`)
  const closed = await read(`/api/budgets/${previous}`), next = await read(`/api/budgets/${current}`)
  assert.equal(closed.closed,true)
  assert.equal(next.ready_to_assign.amount,'700')
  assert.equal(next.categories.find(c=>c.id===carryCategory.id).remaining.amount,'200')
  checks.push('complete ended month closes with release and carry; next period refetch matches checkpoint')
  await b.evaluate(`(()=>{const e=document.querySelector('#budget-period');e.value=${json(previous)};e.dispatchEvent(new Event('change',{bubbles:true}))})()`)
  await b.waitFor(`Boolean(document.querySelector('#assign-${category.id}')?.closest('form').querySelector('textarea'))`)
  await b.set(assignmentSelector,'750')
  const reasonSelector = `${assignmentSelector} ~ .reason-field textarea`
  assert.equal(await b.evaluate(`document.querySelector(${json(reasonSelector)}).required`),true)
  await b.set(reasonSelector,'Correct closed assignment intent after review')
  await submitAssignment(category.id)
  await b.waitFor(`fetch('/api/budgets/${previous}').then(r=>r.json()).then(p=>p.categories.find(c=>c.id===${json(category.id)}).assignment_intent==='750')`)
  const corrected = await read(`/api/budgets/${previous}`)
  assert.equal(corrected.categories.find(c=>c.id===category.id).assigned,'700')
  for(const field of ['funding','ready_to_assign','activity']) assert.deepEqual(corrected[field],closed[field])
  assert.match(await b.evaluate(`document.querySelector(${json(assignmentSelector)}).closest('tr').innerText`),/Objetivo vigente/)
  checks.push('closed assignment correction requires reason and preserves closed assigned value/snapshot totals')
  console.log(JSON.stringify({result:'PASS',resume,resumePhase,origin,period:previous,checks,categoryId:category.id,expenseId:expense.resource_id,closedReadyToAssign:closed.ready_to_assign,nextReadyToAssign:next.ready_to_assign}))
} catch (error) {
  console.error(JSON.stringify({budgetTrace:await b.evaluate('window.budgetTrace ?? []')}))
  throw error
} finally { await b.close() }
