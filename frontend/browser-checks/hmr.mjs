import assert from 'node:assert/strict'
import { readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { browser } from './chromium.mjs'
const source=fileURLToPath(new URL('../app/src/App.vue',import.meta.url))
const original=await readFile(source,'utf8')
const b=await browser()
try{
 await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
 await b.evaluate(`(async()=>{const {createHotContext}=await import('/@vite/client');createHotContext('/__browser_verification__').on('vite:afterUpdate',()=>window.__verificationHotUpdate=true)})()`)
 await writeFile(source,original.replace('</style>','\n:root { --browser-verification: 1; }\n</style>'))
 await b.waitFor(`window.__verificationHotUpdate===true`)
 assert.equal(await b.evaluate(`document.querySelector('#accounts-title').innerText`),'Cuentas')
 console.log(JSON.stringify({result:'PASS',checks:['Vite websocket delivered actual file update','account page remains usable after HMR']}))
}finally{await writeFile(source,original);await b.close()}
