// Read-only shell regressions against the real application; no financial writes.
import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'

const b = await browser()
const checks = []
try {
  await b.command('Page.enable')
  await b.command('Page.navigate', { url: `${b.origin}/?dashboardPeriod=2026-03` })
  await b.waitFor('Boolean(document.querySelector("#home-title")) && document.querySelector("#dashboard-period")?.value === "2026-03"')
  assert.ok(await b.evaluate('document.querySelector(".dashboard-heading").textContent.includes("2026-03")'))
  assert.equal(await b.evaluate('document.querySelector("nav a[aria-current=page]")?.textContent'), 'Panel')
  assert.ok(await b.evaluate('document.querySelector(".current-period").textContent.includes("marzo")'))
  checks.push('direct dashboardPeriod link selects dashboard and shell period')

  await b.command('Page.navigate', { url: `${b.origin}/budget?period=2026-02&dashboardPeriod=2026-03` })
  await b.waitFor('document.querySelector("#budget-period")?.value === "2026-02" && document.querySelector("#dashboard-period")?.value === "2026-03"')
  assert.equal(await b.evaluate('document.querySelector("nav a[aria-current=page]")?.textContent'), 'Presupuesto')
  assert.ok(await b.evaluate('document.querySelector("main .page-heading").textContent.includes("2026-02")'))
  checks.push('budget period and dashboard period read independent URL parameters')

  // The existing shell control navigates home and owns only dashboardPeriod.
  await b.evaluate(`(() => { const input = document.querySelector('#dashboard-period'); input.value = '2026-04'; input.dispatchEvent(new Event('change', { bubbles: true })) })()`)
  await b.waitFor('location.pathname === "/" && new URLSearchParams(location.search).get("dashboardPeriod") === "2026-04" && Boolean(document.querySelector("#home-title"))')
  assert.equal(await b.evaluate('new URLSearchParams(location.search).has("period")'), false)
  assert.ok(await b.evaluate('document.querySelector(".dashboard-heading").textContent.includes("2026-04")'))
  assert.equal(await b.evaluate('document.querySelector("nav a[aria-current=page]")?.textContent'), 'Panel')
  checks.push('changing shell period routes home with dashboardPeriod, preserving URL contract')

  await b.command('Page.navigate', { url: `${b.origin}/accounts?dashboardPeriod=2026-03` })
  await b.waitFor('Boolean(document.querySelector("#account-name"))')
  assert.equal(await b.evaluate('document.querySelector("nav a[aria-current=page]")?.textContent'), 'Cuentas')
  await b.click('nav a[href="/?dashboardPeriod=2026-03"]')
  await b.waitFor('Boolean(document.querySelector("#home-title")) && document.querySelector("#dashboard-period")?.value === "2026-03"')
  checks.push('active navigation follows routes and Panel link preserves selected dashboard period')

  for (const width of [1440, 1024, 390, 320]) {
    await b.command('Emulation.setDeviceMetricsOverride', { width, height: 960, deviceScaleFactor: 1, mobile: false })
    assert.ok(await b.evaluate(`Array.from(document.querySelector('.app-header').querySelectorAll('.brand, .current-period, .period-picker, nav, .appearance-menu')).every(e => { const r = e.getBoundingClientRect(); return r.left >= 0 && r.right <= innerWidth + 1 && r.width > 0 })`), `Shell controls fit at ${width}px`)
  }
  checks.push('shell controls remain visible within desktop and narrow viewport widths')
  console.log(JSON.stringify({ result: 'PASS', browser: await b.command('Browser.getVersion'), checks }, null, 2))
} finally {
  await b.close()
}
