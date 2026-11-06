import assert from 'node:assert/strict'
import { readFile, writeFile } from 'node:fs/promises'
import { browser } from './chromium.mjs'
const b=await browser()
const snapshotFile=process.argv[3] ?? '/tmp/lukas-browser-account-snapshot.json'
try{
 await b.waitFor(`Boolean(document.querySelector('#account-name'))`)
 const accounts=await b.evaluate(`fetch('/api/accounts').then(r=>r.json())`)
 if(process.argv[2]==='snapshot'){
  const result=await b.evaluate(`(async()=>{const g=await fetch('/api/bootstrap').then(r=>r.json());const wallets=[{currency:'CLP',opening_amount:'100',opening_date:'2026-11-05'},{currency:'USD',opening_amount:'2',opening_date:'2026-11-05'}];const r=await fetch('/api/accounts',{method:'POST',headers:{'Content-Type':'application/json','Idempotency-Key':crypto.randomUUID(),'Installation-Generation':g.installation_generation},body:JSON.stringify({name:'Browser atomic failure should not exist',account_type:'bank',base_currency:'CLP',budget_eligible:true,wallets})});return {status:r.status,body:await r.json()}})()`)
  assert.equal(result.status,422)
  assert.deepEqual(await b.evaluate(`fetch('/api/accounts').then(r=>r.json())`),accounts)
  await writeFile(snapshotFile,JSON.stringify(accounts))
  console.log(JSON.stringify({result:'PASS',check:'atomic rejected multi-wallet bank',accounts:accounts.length,snapshotFile}))
 }else{
  assert.deepEqual(accounts,JSON.parse(await readFile(snapshotFile,'utf8')))
  console.log(JSON.stringify({result:'PASS',check:'all account details persist unchanged after API restart',accounts:accounts.length}))
 }
}finally{await b.close()}
