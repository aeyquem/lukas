// Firefox/Zen functional smoke via WebDriver BiDi.
// Protocol reference: https://developer.mozilla.org/en-US/docs/Web/WebDriver/How_to/Create_BiDi_connection
import assert from 'node:assert/strict'
import { spawn } from 'node:child_process'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
const origin=process.env.LUKAS_BROWSER_ORIGIN ?? 'http://localhost:4174'
assert.equal(origin,'http://localhost:4174','Use the isolated verification database')
const expectedGeneration=process.env.LUKAS_BROWSER_GENERATION
assert.match(expectedGeneration??'',/^[0-9a-f-]{36}$/i,'Provide the isolated database generation from its PostgreSQL workspace row')
const executable=process.env.LUKAS_COMPAT_BROWSER ?? 'firefox'
assert.ok(['firefox','zen-browser'].includes(executable))
const profile=await mkdtemp(join(tmpdir(),'lukas-compat-'))
// Use a disposable automation profile without changing browser preferences.
const child=spawn(executable,['--headless','--no-remote','--profile',profile,'--remote-debugging-port','0','about:blank'],{stdio:['ignore','ignore','pipe']})
let stderr='', ws, browserVersion
const pause=ms=>new Promise(r=>setTimeout(r,ms))
child.stderr.on('data',chunk=>{stderr+=chunk})
try{
 let endpoint
 for(let i=0;i<120;i++){endpoint=stderr.match(/WebDriver BiDi listening on (ws:\/\/[^\s]+)/)?.[1];if(endpoint)break;if(child.exitCode!==null)throw new Error(`Browser exited ${child.exitCode}: ${stderr}`);await pause(250)}
 if(!endpoint)throw new Error(`No BiDi endpoint: ${stderr}`)
 ws=new WebSocket(`${endpoint.replace(/\/$/,'')}/session`)
 await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject})
 let id=0;const pending=new Map()
 ws.onmessage=e=>{const m=JSON.parse(e.data);if(!m.id)return;const entry=pending.get(m.id);if(!entry)return;pending.delete(m.id);clearTimeout(entry.timer);m.type==='error'?entry.reject(new Error(`${m.error}: ${m.message}`)):entry.resolve(m.result)}
 const command=(method,params={})=>new Promise((resolve,reject)=>{const n=++id;const timer=setTimeout(()=>{pending.delete(n);reject(new Error(`Timed out ${method}`))},20000);pending.set(n,{resolve,reject,timer});ws.send(JSON.stringify({id:n,method,params}))})
 const session=await command('session.new',{capabilities:{alwaysMatch:{}}})
 browserVersion=session.capabilities
 const {context}=await command('browsingContext.create',{type:'tab'})
 const evaluate=async expression=>{const result=await command('script.evaluate',{expression:`(async()=>{const value=await (${expression});return JSON.stringify(value??null)})()`,target:{context},awaitPromise:true});if(result.type==='exception')throw new Error(JSON.stringify(result.exceptionDetails));return JSON.parse(result.result.value)}
 const waitFor=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await pause(100)}throw new Error(`Timed out ${expression}; ${await evaluate('document.body.innerText')}`)}
 await command('browsingContext.activate',{context})
 await command('browsingContext.navigate',{context,url:`${origin}/accounts`,wait:'complete'})
 await waitFor(`Boolean(document.querySelector('#account-name'))`)
 const bootstrap=await evaluate(`fetch('/api/bootstrap').then(r=>r.json())`)
 assert.equal(bootstrap.installation_generation,expectedGeneration,'Verification proxy targets a different database; refusing account creation')
 const name=`Browser ${executable} functional ${Date.now()}`
 await evaluate(`(()=>{const e=document.querySelector('#account-name');e.value=${JSON.stringify(name)};e.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('form button[type="submit"]').click()})()`)
 await waitFor(`Boolean(document.querySelector('.success-notice'))`)
 const account=await evaluate(`fetch('/api/accounts').then(r=>r.json()).then(a=>a.find(x=>x.name===${JSON.stringify(name)}))`)
 assert.equal(account.wallets[0].balance,'0')
 const headings=[]
 for(const route of ['transactions','budget','accounts']){
  await command('browsingContext.navigate',{context,url:`${origin}/${route}`,wait:'complete'})
  await waitFor(`location.pathname==='/${route}'`)
  const expected={transactions:'Movimientos',budget:'Presupuesto',accounts:'Cuentas'}[route]
  await waitFor(`document.querySelector('main h1, .page-heading h1')?.innerText===${JSON.stringify(expected)}`)
  await evaluate('window.__beforeReload = true')
  await command('browsingContext.reload',{context,wait:'complete'})
  await waitFor('window.__beforeReload !== true')
  await waitFor(`document.querySelector('main h1, .page-heading h1')?.innerText===${JSON.stringify(expected)}`)
  headings.push(await evaluate(`document.querySelector('main h1, .page-heading h1').innerText`))
 }
 await command('browsingContext.setViewport',{context,viewport:{width:390,height:844,devicePixelRatio:1}})
 assert.equal(await evaluate('document.documentElement.scrollWidth <= window.innerWidth'),true,'Account route should fit a 390px viewport')
 console.log(JSON.stringify({result:'PASS',executable,browserVersion,checks:['account create and API readback','route navigation and refresh','390px viewport'],headings,accountId:account.id}))
 await command('session.end')
}finally{
 ws?.close();child.kill();
 await Promise.race([new Promise(resolve=>child.once('exit',resolve)),pause(3000)])
 if(child.exitCode===null)child.kill('SIGKILL')
 await pause(100);await rm(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100})
}
