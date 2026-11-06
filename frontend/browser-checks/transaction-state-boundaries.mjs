// Mounted form must retain wallet, chronology choice and draft through account refetch.
import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'
const b = await browser()
try {
 await b.command('Page.navigate',{url:`${b.origin}/transactions`})
 await b.waitFor(`document.querySelector('#transaction-wallet')?.options.length > 1`)
 const wallet = await b.evaluate(`fetch('/api/accounts').then(r=>r.json()).then(accounts=>accounts.find(a=>a.name.startsWith('Browser same-day opener ')).wallets[0].id)`)
 await b.set('#transaction-kind','income')
 await b.set('#transaction-wallet',wallet)
 await b.set('#transaction-amount','37')
 await b.set('#transaction-note','Dirty transaction survives account refresh')
 await b.click('#wallet-opening-treatment-apply')
 const before = await b.evaluate(`({wallet:document.querySelector('#transaction-wallet').value,amount:document.querySelector('#transaction-amount').value,note:document.querySelector('#transaction-note').value,opening:document.querySelector('#wallet-opening-treatment-apply').checked})`)
 await b.evaluate(`(async()=>{const {useFinanceInvalidation}=await import('/src/features/accounts/api/mutations.ts');const app=document.querySelector('#app').__vue_app__;await app.runWithContext(()=>useFinanceInvalidation())()})()`)
 const after = await b.evaluate(`({wallet:document.querySelector('#transaction-wallet').value,amount:document.querySelector('#transaction-amount').value,note:document.querySelector('#transaction-note').value,opening:document.querySelector('#wallet-opening-treatment-apply').checked})`)
 assert.deepEqual(after,before)
 console.log(JSON.stringify({result:'PASS',checks:['mounted account-query invalidation preserves dirty transaction amount/note/wallet/opening choice']}))
}finally{await b.close()}
