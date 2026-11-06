// Read-only UI check against the running development stack. No financial writes.
import assert from 'node:assert/strict'
import { writeFile } from 'node:fs/promises'
import { browser } from './chromium.mjs'

const b = await browser()
const results = []
try {
  await b.command('Page.enable')
  for (const width of [1440, 1024, 768, 650, 390, 320]) {
    await b.command('Emulation.setDeviceMetricsOverride', { width, height: 960, deviceScaleFactor: 1, mobile: false })
    for (const route of ['/', '/accounts', '/transactions', '/budget']) {
      await b.command('Page.navigate', { url: `${b.origin}${route}` })
      await b.waitFor(`location.pathname === ${JSON.stringify(route)} && Boolean(document.querySelector("main h1"))`)
      await b.waitFor('!document.body.innerText.includes("Cargando")')
      const layout = await b.evaluate(`({ width: innerWidth, content: document.documentElement.scrollWidth, title: document.querySelector('main h1').textContent, background: getComputedStyle(document.querySelector('.app-header')).backgroundColor })`)
      assert.ok(layout.content <= width + 1, `${route} overflows at ${width}: ${JSON.stringify(layout)}`)
      results.push({ route, ...layout })
    }
  }
  await b.command('Emulation.setDeviceMetricsOverride', { width: 1440, height: 1000, deviceScaleFactor: 1, mobile: false })
  await b.command('Page.navigate', { url: `${b.origin}/` })
  await b.waitFor('Boolean(document.querySelector("#home-title"))')
  const forestAccent = await b.evaluate('getComputedStyle(document.documentElement).getPropertyValue("--accent")')
  const shot = await b.command('Page.captureScreenshot', { format: 'png' })
  await writeFile('/tmp/lukas-forest-desktop.png', Buffer.from(shot.data, 'base64'))
  await b.click('.appearance-menu summary')
  await b.set('#theme-select', 'lagoon')
  assert.equal(await b.evaluate('document.documentElement.dataset.theme'), 'lagoon')
  assert.notEqual(await b.evaluate('getComputedStyle(document.documentElement).getPropertyValue("--accent")'), forestAccent)
  await b.evaluate('window.__appearanceBeforeReload = true')
  await b.command('Page.reload')
  await b.waitFor('window.__appearanceBeforeReload !== true && document.documentElement.dataset.theme === "lagoon" && Boolean(document.querySelector("#home-title"))')
  await b.command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: false })
  await b.click('.appearance-menu summary')
  assert.ok(await b.evaluate('document.querySelector(".appearance-panel").getBoundingClientRect().right <= innerWidth'))
  await b.set('#theme-select', 'forest')
  await b.click('.appearance-menu summary')
  // Exercise the night theme and actual webfont loading without financial writes.
  await b.click('.appearance-menu summary')
  await b.set('#theme-select', 'nord')
  assert.equal(await b.evaluate('getComputedStyle(document.documentElement).colorScheme'), 'dark')
  await b.evaluate('window.__appearanceBeforeReload = true')
  await b.command('Page.reload')
  await b.waitFor('window.__appearanceBeforeReload !== true && document.documentElement.dataset.theme === "nord" && Boolean(document.querySelector("#home-title"))')
  for (const weight of [400, 600]) {
    assert.equal(await b.evaluate(`document.fonts.load('${weight} 14px "Lukas Fira Code"', '0123456789 CLP $ − ñ').then(fonts => fonts.length)`), 1)
  }
  for (const route of ['/accounts', '/transactions', '/budget']) {
    await b.command('Page.navigate', { url: `${b.origin}${route}` })
    await b.waitFor(`location.pathname === ${JSON.stringify(route)} && Boolean(document.querySelector("main h1")) && !document.body.innerText.includes("Cargando")`)
    assert.ok(await b.evaluate('document.documentElement.scrollWidth <= innerWidth + 1'), `Nord mobile overflow: ${route}`)
    const input = await b.evaluate(`(() => { const e = document.querySelector('input[inputmode="decimal"], input[type="month"]'); return e ? getComputedStyle(e).fontFamily : null })()`)
    if (input) assert.ok(input.includes('Lukas Fira Code'), `Numeric font missing: ${route}`)
  }
  await b.command('Page.navigate', { url: `${b.origin}/` })
  await b.waitFor('Boolean(document.querySelector("#home-title"))')
  const nordShot = await b.command('Page.captureScreenshot', { format: 'png' })
  await writeFile('/tmp/lukas-nord-mobile.png', Buffer.from(nordShot.data, 'base64'))
  await b.click('.appearance-menu summary')
  await b.set('#theme-select', 'forest')
  await b.click('.appearance-menu summary')
  const mobileShot = await b.command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true })
  await writeFile('/tmp/lukas-forest-mobile.png', Buffer.from(mobileShot.data, 'base64'))
  console.log(JSON.stringify({ result: 'PASS', browser: await b.command('Browser.getVersion'), checks: ['24 route/viewport layouts', 'theme changes palette', 'theme survives reload', 'mobile appearance menu fits', 'Nord dark scheme and persistence', 'bundled numeric webfonts load', 'Nord financial screens fit mobile'], layouts: results, screenshots: ['/tmp/lukas-forest-desktop.png', '/tmp/lukas-forest-mobile.png'] }, null, 2))
} finally {
  await b.close()
}
