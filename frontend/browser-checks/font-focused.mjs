import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'

const b = await browser()
try {
  await b.waitFor('Boolean(document.querySelector("#wallet-amount-0"))')
  const input = await b.evaluate('getComputedStyle(document.querySelector("#wallet-amount-0")).fontFamily')
  const regular = await b.evaluate('document.fonts.load("400 14px \'Lukas Fira Code\'", "0123456789 CLP $ − ñ").then(fonts => fonts.length)')
  const semibold = await b.evaluate('document.fonts.load("600 14px \'Lukas Fira Code\'", "0123456789 CLP $ − ñ").then(fonts => fonts.length)')
  assert.match(input, /Lukas Fira Code/)
  assert.equal(regular, 1)
  assert.equal(semibold, 1)
  console.log(JSON.stringify({ result: 'PASS', inputFont: input, regular, semibold }))
} finally {
  await b.close()
}
