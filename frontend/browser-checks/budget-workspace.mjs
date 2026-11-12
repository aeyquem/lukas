// Destructive only to a fresh, disposable verification database behind port 4174.
// Browser -> same-origin proxy -> real Rust API -> PostgreSQL; no mocked API values.
import assert from 'node:assert/strict'
import { writeFile } from 'node:fs/promises'
import { browser } from './chromium.mjs'

const origin = process.env.LUKAS_BROWSER_ORIGIN
assert.equal(origin, 'http://localhost:4174', 'Workspace checks require the isolated verification proxy')
const b = await browser()
const checks = []
const q = JSON.stringify
const expressionForButton = name => `Array.from(document.querySelectorAll('button')).find(e=>e.getClientRects().length && (e.getAttribute('aria-label') ?? e.textContent.trim())===${q(name)})`
const button = async name => {
  const expression = expressionForButton(name)
  await b.waitFor(`Boolean(${expression}) && !(${expression}).disabled`)
  await b.evaluate(`(()=>{const e=${expression};e.focus();e.click()})()`)
}
const editAssignment = async id => {
  if (!await b.evaluate(`Boolean(document.querySelector('#assign-${id}'))`)) {
    await b.click(`[data-category-id="${id}"] .assigned-value`)
  }
}
const set = async (selector, value) => {
  if (selector.startsWith('#assign-')) await editAssignment(selector.slice('#assign-'.length))
  await b.waitFor(`Boolean(document.querySelector(${q(selector)})) && document.querySelector(${q(selector)}).getClientRects().length>0 && !document.querySelector(${q(selector)}).disabled`)
  await b.set(selector, value)
}
const press = async (selector, key = 'Enter') => {
  await b.waitFor(`Boolean(document.querySelector(${q(selector)})) && !document.querySelector(${q(selector)}).disabled`)

  await b.evaluate(`document.querySelector(${q(selector)}).focus()`)
  await b.command('Input.dispatchKeyEvent', { type: 'keyDown', key, code: key, windowsVirtualKeyCode: key === 'Enter' ? 13 : 27, ...(key === 'Enter' ? { text: '\r', unmodifiedText: '\r' } : {}) })
  await b.command('Input.dispatchKeyEvent', { type: 'keyUp', key, code: key, windowsVirtualKeyCode: key === 'Enter' ? 13 : 27 })
}
const installApi = async () => b.evaluate(`
window.workspaceRead = async path => { const r=await fetch(path); const body=await r.json(); if(!r.ok)throw Error(JSON.stringify({status:r.status,body})); return body };
window.workspaceWrite = async(method,path,body) => {
  const bootstrap=await window.workspaceRead('/api/bootstrap');
  const r=await fetch(path,{method,headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':bootstrap.installation_generation},body:JSON.stringify(body)});
  const result=await r.json(); if(!r.ok)throw Error(JSON.stringify({status:r.status,result})); return result;
}`)
const read = path => b.evaluate(`window.workspaceRead(${q(path)})`)
const write = (method, path, body) => b.evaluate(`window.workspaceWrite(${q(method)},${q(path)},${q(body)})`)
const refresh = async () => {
  await button('Actualizar')
  await b.waitFor(`!document.body.innerText.includes('Cargando presupuesto…')`)
}
const snapshot = period => read(`/api/budgets/${period}`)
const assigned = async (period, categoryId, amount) => {
  const before = await snapshot(period)
  const category = [...before.categories, ...(before.archived_categories ?? [])].find(c => c.id === categoryId)
  return write('PUT', `/api/budgets/${period}/assignments/${categoryId}`, {
    amount, expected_period_revision: before.revision, expected_revision: category.assignment_revision,
  })
}
const waitAssigned = (period, id, amount) => b.waitFor(`window.workspaceRead('/api/budgets/${period}').then(p=>p.categories.find(c=>c.id===${q(id)})?.assigned===${q(amount)})`)
const selectPeriod = async period => {
  await set('#budget-period', period)
  await b.evaluate(`document.querySelector('#budget-period').dispatchEvent(new Event('change',{bubbles:true}))`)
  await b.waitFor(`new URL(location.href).searchParams.get('period')===${q(period)} && !document.body.innerText.includes('Cargando presupuesto…')`)
}
const assertSingleTable = async () => {
  assert.equal(await b.evaluate(`Array.from(document.querySelectorAll('table')).filter(e=>e.getClientRects().length).length`), 1)
  assert.equal(await b.evaluate(`(()=>{const t=Array.from(document.querySelectorAll('table')).find(e=>e.getClientRects().length);return t.getAttribute('aria-label')??document.getElementById(t.getAttribute('aria-labelledby'))?.textContent.trim()??t.querySelector('caption')?.textContent.trim()})()`), 'Presupuesto')
  assert.equal(await b.evaluate(`Boolean(Array.from(document.querySelectorAll('h2')).find(e=>e.getClientRects().length&&e.textContent.trim()==='Administrar categorías'))`), false)
}

try {
  await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
  await installApi()
  await b.command('Emulation.setDeviceMetricsOverride', { width: 1440, height: 1000, deviceScaleFactor: 1, mobile: false })
  assert.deepEqual(await read('/api/accounts'), [], 'Use a fresh isolated database; this check retains fixtures and closes a period')
  const current = await b.evaluate(`new Intl.DateTimeFormat('en-CA',{timeZone:'America/Santiago',year:'numeric',month:'2-digit'}).format(new Date())`)
  const [year, month] = current.split('-').map(Number)
  const previous = month === 1 ? `${year - 1}-12` : `${year}-${String(month - 1).padStart(2, '0')}`
  await write('POST', '/api/accounts', { name: 'Workspace verification bank', account_type: 'bank', base_currency: 'CLP', budget_eligible: true, wallets: [{ currency: 'CLP', opening_amount: '1000', opening_date: `${previous}-01` }] })
  await write('POST', '/api/accounts', { name: 'Workspace verification card', account_type: 'card', base_currency: 'CLP', budget_eligible: true, wallets: [{ currency: 'USD', opening_amount: '0', opening_date: `${previous}-01` }] })
  const accounts = await read('/api/accounts')
  const wallet = accounts.find(a => a.name === 'Workspace verification bank').wallets[0].id
  const foreignWallet = accounts.find(a => a.name === 'Workspace verification card').wallets[0].id
  await write('POST', '/api/transactions', { kind: 'income', wallet_id: wallet, amount: '200', business_date: `${previous}-02`, note: 'Workspace verification income' })

  await b.command('Page.navigate', { url: `${origin}/budget?period=${previous}` })
  await b.waitFor(`Boolean(document.querySelector('#budget-period')) && document.querySelectorAll('table').length===1`)
  await installApi()
  await assertSingleTable()
  assert.equal(await b.evaluate(`Boolean(document.querySelector('.category-name[draggable="true"]'))`), true, 'Category dragging is available directly')
  await button('Administrar categorías')
  await b.waitFor(`Boolean(document.querySelector('.category-manager'))`)
  checks.push('Presupuesto contains one named table; administration is a separate tab')

  // Remaining interaction cases use the semantic names defined by the feature.
  const field = async (name, value) => {
    const selector = await b.evaluate(`(()=>{const direct=Array.from(document.querySelectorAll('input,select,textarea')).find(e=>e.getClientRects().length&&e.getAttribute('aria-label')===${q(name)});const label=Array.from(document.querySelectorAll('label')).find(e=>e.getClientRects().length&&Array.from(e.childNodes).filter(n=>n.nodeType===Node.TEXT_NODE).map(n=>n.textContent).join('').trim()===${q(name)});const e=direct??(label?.htmlFor?document.getElementById(label.htmlFor):label?.querySelector('input,select,textarea'));if(!e)throw Error('Missing field: '+${q(name)});if(!e.id)e.id='workspace-field-'+crypto.randomUUID();return '#'+e.id})()`)
    await set(selector, value)
  }
  const groupName = 'Workspace verification group'
  await field('Nombre del grupo', groupName)
  await button('Añadir grupo')
  await b.waitFor(`window.workspaceRead('/api/budget-groups').then(a=>a.some(g=>g.name===${q(groupName)}))`)
  let customGroup = (await read('/api/budget-groups')).find(g => g.name === groupName)
  const renamedGroup = `${groupName} renamed`
  await field(`Nombre del grupo ${groupName}`, renamedGroup)
  await button(`Guardar grupo ${groupName}`)
  await b.waitFor(`window.workspaceRead('/api/budget-groups').then(a=>a.some(g=>g.id===${q(customGroup.id)}&&g.name===${q(renamedGroup)}))`)
  customGroup = (await read('/api/budget-groups')).find(g => g.id === customGroup.id)
  const createCategory = async name => {
    await button('Añadir categoría')
    await b.waitFor(`Boolean(document.querySelector('dialog[open]'))`)
    await field('Nombre de categoría', name)
    await field('Grupo de categoría', customGroup.id)
    await button('Guardar categoría')
    await b.waitFor(`window.workspaceRead('/api/categories').then(a=>a.some(c=>c.name===${q(name)})) && !document.querySelector('dialog[open]')`)
    return (await read('/api/categories')).find(c => c.name === name)
  }
  const releaseCategory = await createCategory('Workspace release category')
  const carryCategory = await createCategory('Workspace carry category')
  const unusedCategory = await createCategory('Workspace disposable category')
  checks.push('group create/rename and category creation through separate administration tab and drawer')

  await button(`Editar ${unusedCategory.name}`)
  await b.waitFor(`Boolean(document.querySelector('dialog[open]'))`)
  const deleteButton = expressionForButton('Eliminar permanentemente')
  assert.equal(await b.evaluate(`(${deleteButton}).disabled`), true, 'Deletion requires explicit confirmation')
  await b.evaluate(`Array.from(document.querySelectorAll('dialog label')).find(e=>e.textContent.includes('Confirmar eliminación permanente')).querySelector('input').click()`)
  await button('Eliminar permanentemente')
  await b.waitFor(`window.workspaceRead('/api/categories').then(a=>!a.some(c=>c.id===${q(unusedCategory.id)})) && !document.querySelector('dialog[open]')`)
  checks.push('eligible permanent category deletion requires explicit confirmation and refreshes management')

  await button('Presupuesto')
  await assertSingleTable()
  const inputFor = id => `#assign-${id}`
  await set(inputFor(releaseCategory.id), '300')
  await press(inputFor(releaseCategory.id))
  await waitAssigned(previous, releaseCategory.id, '300')
  await b.waitFor(`!document.body.innerText.includes('Cargando presupuesto…')`)
  await set(inputFor(carryCategory.id), '200')
  await press(inputFor(carryCategory.id))
  await waitAssigned(previous, carryCategory.id, '200')
  await write('POST', '/api/transactions', { kind: 'expense', wallet_id: wallet, amount: '100', business_date: `${previous}-03`, category_id: releaseCategory.id, note: 'Workspace release category history' })
  await refresh()
  const beforeClose = await snapshot(previous)
  assert.equal(beforeClose.ready_to_assign.amount, '700')
  assert.equal(beforeClose.available_to_budget_breakdown.income.amount, '200')
  assert.equal(beforeClose.available_to_budget_breakdown.openings.amount, '1000')
  assert.equal(beforeClose.available_to_budget_breakdown.category_rollover.amount, '0')
  checks.push('inline Enter assignment saves exact decimal strings and authoritative totals distinguish income and opening funding')

  await button('Editar organización')
  await b.waitFor(`Boolean(document.querySelector('[draggable="true"]'))`)
  const orderBefore = (await read('/api/categories')).find(c => c.id === releaseCategory.id).sort_order
  await button(`Bajar categoría ${releaseCategory.name}`)
  await b.waitFor(`window.workspaceRead('/api/categories').then(a=>a.find(c=>c.id===${q(releaseCategory.id)}).sort_order!==${orderBefore})`)
  // Drag the custom group before the first seeded group using actual browser drag events.
  const groupsBefore = await read('/api/budget-groups')
  const firstGroup = groupsBefore.filter(g => !g.archived).sort((a, z) => a.sort_order - z.sort_order)[0]
  await b.evaluate(`(()=>{const source=Array.from(document.querySelectorAll('button')).find(e=>e.getAttribute('aria-label')===${q(`Arrastrar grupo ${renamedGroup}`)});const target=Array.from(document.querySelectorAll('button')).find(e=>e.getAttribute('aria-label')===${q(`Arrastrar grupo ${firstGroup.name}`)});const dataTransfer=new DataTransfer();source.dispatchEvent(new DragEvent('dragstart',{bubbles:true,dataTransfer}));target.dispatchEvent(new DragEvent('dragover',{bubbles:true,cancelable:true,dataTransfer}));target.dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer}));source.dispatchEvent(new DragEvent('dragend',{bubbles:true,dataTransfer}));})()`)
  await b.waitFor(`window.workspaceRead('/api/budget-groups').then(a=>a.find(g=>g.id===${q(customGroup.id)}).sort_order!==${customGroup.sort_order})`)
  await set(`#move-${carryCategory.id}`, firstGroup.id)
  await b.waitFor(`window.workspaceRead('/api/categories').then(a=>a.find(c=>c.id===${q(carryCategory.id)}).group_id===${q(firstGroup.id)})`)
  await button('Terminar edición')
  assert.equal(await b.evaluate(`Boolean(document.querySelector('.drag-handle[draggable="true"]'))`), false)
  await refresh()
  assert.equal(await b.evaluate(`Boolean(document.querySelector('.drag-handle[draggable="true"]'))`), false)
  checks.push('keyboard category order, cross-group move, and group drag persist; group arrangement locks after edit mode and refresh')
  await button('Administrar categorías')
  await b.evaluate(`(()=>{const original=window.fetch.bind(window);window.groupArchiveStatus=null;window.fetch=async(url,options)=>{const r=await original(url,options);if(String(url).endsWith('/api/budget-groups/${customGroup.id}')&&options?.method==='PUT')window.groupArchiveStatus=r.status;return r}})()`)
  await button(`Archivar grupo ${renamedGroup}`)
  await b.waitFor(`window.groupArchiveStatus!==null`)
  assert.equal(await b.evaluate('window.groupArchiveStatus'), 409)
  assert.equal((await read('/api/budget-groups')).find(g=>g.id===customGroup.id).archived, false)
  await button('Presupuesto')
  checks.push('group containing an active category rejects archival and retains the visible group')

  await button('Ver vista previa del cierre')
  await b.waitFor(`Boolean(document.querySelector('.close-preview'))`)
  await b.click(`input[name="disposition-${releaseCategory.id}"][value="release"]`)
  await button('Confirmar cierre')
  await b.waitFor(`document.querySelector('#budget-period')?.value===${q(current)} && !document.body.innerText.includes('Cargando presupuesto…')`)
  const closed = await snapshot(previous)
  const next = await snapshot(current)
  assert.equal(closed.closed, true)
  assert.equal(next.ready_to_assign.amount, '900')
  assert.equal(next.available_to_budget_breakdown.prior_unassigned.amount, '700')
  assert.equal(next.available_to_budget_breakdown.category_releases.amount, '200')
  assert.equal(next.available_to_budget_breakdown.category_rollover.amount, '0')
  assert.equal(next.categories.find(c => c.id === carryCategory.id).remaining.amount, '200')
  await b.waitFor(`Boolean(document.querySelector('details.calculation[open]'))`)
  const explanation = await b.evaluate(`document.querySelector('details[open]').innerText`)
  assert.match(explanation, /700/)
  assert.match(explanation, /200/)
  assert.match(explanation, /arrastre|Arrastre/)
  const componentText = label => b.evaluate(`Array.from(document.querySelectorAll('.breakdown > div')).find(e=>e.querySelector('dt')?.textContent.trim()===${q(label)})?.querySelector('dd')?.innerText`)
  assert.match(await componentText('Arrastre entre categorías'), /CLP\s+0(?:\D|$)/)
  assert.match(await componentText('Saldos liberados al cierre'), /200/)
  assert.match(await componentText('Sin asignar del período anterior'), /700/)
  await writeFile('/tmp/lukas-workspace-desktop.png', Buffer.from((await b.command('Page.captureScreenshot', { format: 'png' })).data, 'base64'))
  checks.push('real close releases 200 separately from 700 prior unassigned; category carry retains 200 while explanation rollover is zero')

  // A refetch must not silently replace a draft's expected revisions.
  await set(inputFor(carryCategory.id), '30.125')
  await assigned(current, carryCategory.id, '20')
  await refresh()
  assert.equal(await b.evaluate(`document.querySelector(${q(inputFor(carryCategory.id))}).value`), '30.125')
  await press(inputFor(carryCategory.id))
  await b.waitFor(`document.body.innerText.includes('borrador')`)
  assert.equal((await snapshot(current)).categories.find(c => c.id === carryCategory.id).assigned, '20')
  await button(`Revisé los valores vigentes de ${carryCategory.name}`)
  await press(inputFor(carryCategory.id))
  await waitAssigned(current, carryCategory.id, '30.125')
  checks.push('stale inline draft survives authoritative refresh and requires explicit review before saving')

  // Simulate a committed request whose response is lost; all data still goes to the real API.
  const pendingRoute = `/api/budgets/${current}/assignments/${carryCategory.id}`
  await b.evaluate(`(()=>{const original=window.fetch.bind(window);window.originalWorkspaceFetch=original;window.lostWorkspaceRequest=null;window.fetch=async(url,options)=>{if(String(url).endsWith(${q(pendingRoute)})&&options?.method==='PUT'&&!window.lostWorkspaceRequest){window.lostWorkspaceRequest={body:options.body,key:new Headers(options.headers).get('Idempotency-Key')};const r=await original(url,options);if(!r.ok)return r;throw new TypeError('Verification discarded committed response')}return original(url,options)}})()`)
  await set(inputFor(carryCategory.id), '40')
  await press(inputFor(carryCategory.id))
  await b.waitFor(`document.body.innerText.includes('Comandos pendientes')`)
  await waitAssigned(current, carryCategory.id, '40')
  const committedRevision = (await snapshot(current)).categories.find(c => c.id === carryCategory.id).assignment_revision
  const lostRequest = await b.evaluate('window.lostWorkspaceRequest')
  await b.command('Page.reload')
  await b.waitFor(`document.body.innerText.includes('Comandos pendientes') && Boolean(document.querySelector('[data-category-id="${carryCategory.id}"]'))`)
  await installApi()
  const stored = await b.evaluate(`Object.keys(localStorage).filter(k=>k.startsWith('lukas.pending-financial-write.v1:')).map(k=>JSON.parse(localStorage.getItem(k)))`)
  assert.equal(stored.length, 1)
  assert.equal(stored[0].body, lostRequest.body)
  assert.equal(stored[0].idempotencyKey, lostRequest.key)
  await button('Reintentar comando exacto')
  await b.waitFor(`!document.body.innerText.includes('Comandos pendientes')`)
  assert.equal((await snapshot(current)).categories.find(c => c.id === carryCategory.id).assignment_revision, committedRevision)
  checks.push('committed response loss survives reload and exact retry does not duplicate assignment revision')

  // The foreign original stays visible as incomplete without inventing a CLP zero.
  await write('POST', '/api/transactions', { kind: 'expense', wallet_id: foreignWallet, amount: '10', business_date: `${current}-02`, category_id: carryCategory.id, note: 'Workspace incomplete foreign expense' })
  await refresh()
  const incomplete = await snapshot(current)
  assert.equal(incomplete.categories.find(c => c.id === carryCategory.id).activity.state, 'incomplete')
  assert.equal(incomplete.categories.find(c => c.id === carryCategory.id).remaining.state, 'incomplete')
  await b.waitFor(`/subtotal conocido|incompleto/.test(document.querySelector('[data-category-id="${carryCategory.id}"]')?.innerText ?? '')`)
  assert.match(await b.evaluate(`document.querySelector('[data-category-id="${carryCategory.id}"]').innerText`), /subtotal conocido|incompleto/)
  checks.push('unvalued USD activity preserves incomplete category values in grouped table')

  await button(`Detalles de ${releaseCategory.name}`)
  await b.waitFor(`Boolean(document.querySelector('dialog[open]'))`)
  assert.equal(await b.evaluate(`document.querySelector('dialog[open]').contains(document.activeElement)`), true)
  assert.equal(await b.evaluate(`(${deleteButton}).disabled`), true, 'Transaction/snapshot history forbids permanent deletion')
  await field('Nombre de categoría', `${releaseCategory.name} renamed`)
  await button('Guardar categoría')
  await b.waitFor(`!document.querySelector('dialog[open]')`)
  await button(`Detalles de ${releaseCategory.name} renamed`)
  await button('Archivar categoría')
  await b.waitFor(`window.workspaceRead('/api/categories').then(a=>a.find(c=>c.id===${q(releaseCategory.id)}).archived) && !document.querySelector('dialog[open]')`)
  assert.equal(await b.evaluate(`Boolean(document.querySelector('[data-category-id="${releaseCategory.id}"]'))`), false)
  const protectedClose = await snapshot(previous)
  for (const key of ['funding', 'ready_to_assign', 'activity']) assert.deepEqual(protectedClose[key], closed[key])
  assert.equal(protectedClose.categories.find(c => c.id === releaseCategory.id).name, releaseCategory.name)
  const archivedCurrent = await snapshot(current)
  assert.ok(archivedCurrent.archived_categories.some(c => c.id === releaseCategory.id))
  await button('Administrar categorías')
  await button(`Editar ${releaseCategory.name} renamed`)
  await b.waitFor(`Boolean(document.querySelector('dialog[open]'))`)
  assert.equal(await b.evaluate(`(${deleteButton}).disabled`), true)
  await button('Cerrar detalles')
  checks.push('drawer rename/archive hides active row while transaction references and immutable closed labels/totals survive')

  // Empty group archive is available; a populated group must remain active.
  await field('Nombre del grupo', 'Workspace empty group')
  await button('Añadir grupo')
  await b.waitFor(`window.workspaceRead('/api/budget-groups').then(a=>a.some(g=>g.name==='Workspace empty group'))`)
  await button('Archivar grupo Workspace empty group')
  await b.waitFor(`window.workspaceRead('/api/budget-groups').then(a=>a.some(g=>g.name==='Workspace empty group'&&g.archived))`)
  checks.push('empty group archives through administration and remains available as archived metadata')

  await button('Presupuesto')
  await selectPeriod(previous)
  await editAssignment(carryCategory.id)
  await b.waitFor(`Boolean(document.querySelector(${q(inputFor(carryCategory.id))})?.closest('form').querySelector('textarea'))`)
  await set(inputFor(carryCategory.id), '250')
  const reason = `[data-category-id="${carryCategory.id}"] .reason-field textarea`
  assert.equal(await b.evaluate(`document.querySelector(${q(reason)}).required`), true)
  await set(reason, 'Workspace verification reviewed closed-period assignment')
  await press(inputFor(carryCategory.id))
  await b.waitFor(`window.workspaceRead('/api/budgets/${previous}').then(p=>p.categories.find(c=>c.id===${q(carryCategory.id)}).assignment_intent==='250')`)
  const correctedClose = await snapshot(previous)
  assert.equal(correctedClose.categories.find(c => c.id === carryCategory.id).assigned, '200')
  for (const key of ['funding', 'ready_to_assign', 'activity']) assert.deepEqual(correctedClose[key], closed[key])
  checks.push('closed inline correction requires a reason and preserves immutable assignment and budget totals')
  await selectPeriod(current)
  await b.command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: false })
  await assertSingleTable()
  await button(`Detalles de ${carryCategory.name}`)
  await b.waitFor(`Boolean(document.querySelector('dialog[open]'))`)
  await writeFile('/tmp/lukas-workspace-drawer.png', Buffer.from((await b.command('Page.captureScreenshot', { format: 'png' })).data, 'base64'))
  const mobileLayout = await b.evaluate(`({width:innerWidth,scrollWidth:document.documentElement.scrollWidth,offenders:Array.from(document.querySelectorAll('body *')).filter(e=>{const r=e.getBoundingClientRect();return r.width>0&&(r.right>innerWidth+1||r.left< -1)}).map(e=>({tag:e.tagName,id:e.id,cls:String(e.className).slice(0,100),left:e.getBoundingClientRect().left,right:e.getBoundingClientRect().right,width:e.getBoundingClientRect().width,scrollWidth:e.scrollWidth})).slice(0,12)})`)
  assert.equal(mobileLayout.scrollWidth<=mobileLayout.width, true, `Narrow viewport has no page overflow: ${JSON.stringify(mobileLayout)}`)
  assert.equal(await b.evaluate(`document.querySelector('dialog[open]').getBoundingClientRect().width<=innerWidth`), true)
  await b.command('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 })
  await b.command('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 })
  await b.waitFor(`!document.querySelector('dialog[open]')`)
  await b.waitFor(`(document.activeElement.getAttribute('aria-label')??document.activeElement.textContent.trim())===${q(`Detalles de ${carryCategory.name}`)}`)
  checks.push('390px drawer fits viewport, contains opening focus, closes with Escape, restores trigger focus')
  console.log(JSON.stringify({ result: 'PASS', origin, period: previous, verifiedAt: new Date().toISOString(), browser: (await b.command('Browser.getVersion')).product, checks }))
} finally {
  await b.close()
}
