import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'

const b = await browser()
try {
  const bootstrap = await b.evaluate(`fetch('/api/bootstrap').then(r=>r.json())`)
  assert.ok(bootstrap.installation_generation, 'The API should return an installation generation')

  await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
  const name = `Browser functional smoke ${Date.now()}`
  await b.set('#account-name', name)
  await b.click('form button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('.success-notice'))`)
  const account = await b.evaluate(`fetch('/api/accounts').then(r=>r.json()).then(accounts=>accounts.find(a=>a.name===${JSON.stringify(name)}))`)
  assert.ok(account, 'The created account should be returned by the API')

  const routes = []
  for (const [route, heading] of [['accounts', 'Cuentas'], ['transactions', 'Movimientos'], ['budget', 'Presupuesto']]) {
    await b.evaluate(`location.href=${JSON.stringify(`${b.origin}/${route}`)}`)
    await b.waitFor(`location.pathname===${JSON.stringify(`/${route}`)}`)
    await b.waitFor(`document.querySelector('main h1, .page-heading h1')?.innerText===${JSON.stringify(heading)}`)
    await b.evaluate('window.__beforeReload = true')
    await b.command('Page.reload')
    await b.waitFor('window.__beforeReload !== true')
    await b.waitFor(`document.readyState==='complete'`)
    await b.waitFor(`document.querySelector('main h1, .page-heading h1')?.innerText===${JSON.stringify(heading)}`)
    routes.push(route)
  }

  await b.waitFor(`Boolean(document.querySelector('.category-new-form input:not([type="number"])'))`)
  const categoryName = `Browser functional smoke ${Date.now()}`
  await b.set('.category-new-form input:not([type="number"])', categoryName)
  await b.set('.category-new-form input[type="number"]', '1000')
  await b.click('.category-new-form button')
  await b.waitFor(`fetch('/api/categories').then(r=>r.json()).then(categories=>categories.some(c=>c.name===${JSON.stringify(categoryName)}))`)
  const category = await b.evaluate(`fetch('/api/categories').then(r=>r.json()).then(categories=>categories.find(c=>c.name===${JSON.stringify(categoryName)}))`)
  await b.click(`[data-category-id="${category.id}"] .assigned-value`)
  const assignment = `#assign-${category.id}`
  await b.waitFor(`Boolean(document.querySelector(${JSON.stringify(assignment)}))`)
  await b.set(assignment, '500')
  await b.click(`${assignment} ~ button[type="submit"]`)
  const period = await b.evaluate(`new Intl.DateTimeFormat('en-CA',{timeZone:'America/Santiago',year:'numeric',month:'2-digit'}).format(new Date())`)
  await b.waitFor(`fetch('/api/budgets/${period}').then(r=>r.json()).then(snapshot=>snapshot.categories.some(c=>c.id===${JSON.stringify(category.id)}&&c.assigned==='500'))`)
  await b.click('.close-panel button')
  await b.waitFor(`Boolean(document.querySelector('.close-preview'))`)

  await b.command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true })
  assert.equal(await b.evaluate('document.documentElement.scrollWidth <= window.innerWidth'), true, 'The budget page should fit a 390px viewport')
  console.log(JSON.stringify({ result: 'PASS', browser: await b.evaluate('navigator.userAgent'), routes, accountId: account.id, categoryId: category.id, checks: ['account create and API readback', 'route navigation and refresh', 'budget assignment and close preview', '390px viewport'] }))
} finally {
  await b.close()
}
