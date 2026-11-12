// Read-only UI check against the running development stack. No financial writes.
import assert from 'node:assert/strict'
import { writeFile } from 'node:fs/promises'
import { browser } from './chromium.mjs'

const b = await browser()
const results = []
async function checkLegibility(theme) {
  const metrics = await b.evaluate(`(() => {
    const color = value => {
      const parts = value.match(/[\\d.]+/g)?.map(Number) ?? []
      return { red: parts[0], green: parts[1], blue: parts[2], alpha: parts[3] ?? 1 }
    }
    const luminance = value => {
      const rgb = color(value)
      const linear = channel => { const normalized = channel / 255; return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4 }
      return 0.2126 * linear(rgb.red) + 0.7152 * linear(rgb.green) + 0.0722 * linear(rgb.blue)
    }
    const probe = document.createElement('span')
    probe.style.cssText = 'position:fixed;visibility:hidden;color:var(--text);background:var(--surface);font-size:16px'
    document.body.append(probe)
    const foreground = getComputedStyle(probe).color
    probe.style.color = 'var(--muted)'
    const mutedForeground = getComputedStyle(probe).color
    probe.style.color = 'var(--text)'
    const background = getComputedStyle(probe).backgroundColor
    const surface = color(background)
    const backgroundLight = luminance(background)
    const contrast = value => {
      const textLight = luminance(value)
      return (Math.max(textLight, backgroundLight) + 0.05) / (Math.min(textLight, backgroundLight) + 0.05)
    }
    probe.style.backgroundColor = 'var(--surface-glass)'
    const glassAlpha = color(getComputedStyle(probe).backgroundColor).alpha
    probe.style.backgroundColor = 'var(--canvas-image-wash)'
    const wallpaperWashAlpha = color(getComputedStyle(probe).backgroundColor).alpha
    probe.remove()
    const control = document.querySelector('main input:not([type=checkbox]):not([type=radio]), main select, main textarea')
    const controlAlpha = control ? color(getComputedStyle(control).backgroundColor).alpha : 1
    return { textContrast: contrast(foreground), mutedContrast: contrast(mutedForeground), glassAlpha, wallpaperWashAlpha, controlAlpha, sampledControl: Boolean(control), surfaceAlpha: surface.alpha }
  })()`)
  assert.ok(metrics.textContrast >= 4.5 && metrics.mutedContrast >= 4.5, `${theme} text contrast on financial surfaces is below 4.5:1: ${JSON.stringify(metrics)}`)
  assert.ok(metrics.glassAlpha >= 0.85, `${theme} glass panels are too transparent for financial content: ${JSON.stringify(metrics)}`)
  assert.ok(metrics.wallpaperWashAlpha >= 0.5, `${theme} wallpaper wash is too transparent behind page content: ${JSON.stringify(metrics)}`)
  assert.ok(metrics.surfaceAlpha >= 0.98, `${theme} solid financial surfaces are not opaque: ${JSON.stringify(metrics)}`)
  assert.ok(metrics.controlAlpha >= 0.98, `${theme} financial controls are not opaque: ${JSON.stringify(metrics)}`)
  return metrics
}
try {
  await b.command('Page.enable')
  for (const width of [2560, 1440, 1024, 768, 650, 390, 320]) {
    await b.command('Emulation.setDeviceMetricsOverride', { width, height: 960, deviceScaleFactor: 1, mobile: false })
    for (const route of ['/', '/accounts', '/transactions', '/budget']) {
      await b.command('Page.navigate', { url: `${b.origin}${route}` })
      await b.waitFor(`location.pathname === ${JSON.stringify(route)} && Boolean(document.querySelector("main h1"))`)
      await b.waitFor('!document.body.innerText.includes("Cargando")')
      const layout = await b.evaluate(`(() => { const icon=document.querySelector('.brand-mark'); const style=getComputedStyle(document.body); const position=style.backgroundPosition.split(',').at(-1).trim().split(/\\s+/).map(value=>value==='center'?'50%':value).join(' '); return { width: innerWidth, content: document.documentElement.scrollWidth, title: document.querySelector('main h1').textContent, background: getComputedStyle(document.querySelector('.app-header')).backgroundColor, wallpaper: style.backgroundImage, wallpaperPosition: position, icon: icon?.currentSrc, iconSize: icon?.getBoundingClientRect().width, iconLoaded: Boolean(icon?.complete && icon.naturalWidth) } })()`)
      assert.ok(layout.content <= width + 1, `${route} overflows at ${width}: ${JSON.stringify(layout)}`)
      assert.ok(layout.wallpaper.includes('/torres-del-paine.png'), `${route} is missing the wallpaper: ${JSON.stringify(layout)}`)
      assert.ok(layout.icon?.endsWith('/ignacio-carrera-pinto.svg') && layout.iconLoaded, `${route} is missing the portrait icon: ${JSON.stringify(layout)}`)
      assert.equal(layout.iconSize, width <= 480 ? 42 : 48, `${route} has the wrong portrait size at ${width}`)
      const expectedPosition = width <= 600 ? '82% 50%' : '100% 50%'
      assert.equal(layout.wallpaperPosition, expectedPosition, `${route} has the wrong wallpaper focal position at ${width}`)
      if (route === '/' && width === 2560) {
        const ultrawideShot = await b.command('Page.captureScreenshot', { format: 'png' })
        await writeFile('/tmp/lukas-forest-ultrawide.png', Buffer.from(ultrawideShot.data, 'base64'))
      }
      if (route === '/accounts' && width === 1440) layout.legibility = await checkLegibility('forest')
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
  const lagoonLegibility = await checkLegibility('lagoon')
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
  await b.command('Page.navigate', { url: `${b.origin}/accounts` })
  await b.waitFor('location.pathname === "/accounts" && Boolean(document.querySelector("main h1")) && !document.body.innerText.includes("Cargando")')
  const nordLegibility = await checkLegibility('nord')
  const nordBackground = await b.evaluate('getComputedStyle(document.body).backgroundImage')
  assert.ok(nordBackground.includes('/torres-del-paine.png'))
  await b.command('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-transparency', value: 'reduce' }] })
  const reducedTransparency = await b.evaluate(`(() => { const header=document.querySelector('.app-header'); const probe=document.createElement('span'); probe.style.backgroundColor='var(--surface)'; document.body.append(probe); const surface=getComputedStyle(probe).backgroundColor; probe.remove(); return { background:getComputedStyle(header).backgroundColor, surface, blur:getComputedStyle(header).backdropFilter } })()`)
  assert.equal(reducedTransparency.background, reducedTransparency.surface)
  assert.equal(reducedTransparency.blur, 'none')
  await b.command('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-transparency', value: 'no-preference' }] })
  for (const weight of [400, 600]) {
    assert.equal(await b.evaluate(`document.fonts.load('${weight} 14px "Lukas Fira Code"', '0123456789 CLP $ − ñ').then(fonts => fonts.length)`), 1)
  }
  for (const route of ['/accounts', '/transactions', '/budget']) {
    await b.command('Page.navigate', { url: `${b.origin}${route}` })
    await b.waitFor(`location.pathname === ${JSON.stringify(route)} && Boolean(document.querySelector("main h1")) && !document.body.innerText.includes("Cargando")`)
    assert.ok(await b.evaluate('document.documentElement.scrollWidth <= innerWidth + 1'), `Nord mobile overflow: ${route}`)
    const input = await b.evaluate(`(() => { const e = document.querySelector('main input[inputmode="decimal"], main input[type="month"]'); return e ? getComputedStyle(e).fontFamily : null })()`)
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
  console.log(JSON.stringify({ result: 'PASS', browser: await b.command('Browser.getVersion'), checks: ['28 route/viewport layouts', 'portrait and wallpaper assets load', 'responsive tower focal positioning without viewport gaps', 'text contrast, panel opacity, wallpaper wash, and solid financial controls in Forest, Lagoon, and Nord', 'theme changes palette', 'theme survives reload', 'mobile appearance menu fits', 'Nord dark scheme and persistence', 'reduced-transparency surfaces', 'bundled numeric webfonts load', 'Nord financial screens fit mobile'], legibility: { forest: results.find(layout => layout.route === '/accounts' && layout.width === 1440)?.legibility, lagoon: lagoonLegibility, nord: nordLegibility }, layouts: results, screenshots: ['/tmp/lukas-forest-desktop.png', '/tmp/lukas-forest-ultrawide.png', '/tmp/lukas-forest-mobile.png'] }, null, 2))
} finally {
  await b.close()
}
