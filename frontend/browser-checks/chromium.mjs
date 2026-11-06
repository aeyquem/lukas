import { spawn } from 'node:child_process'
import { mkdtemp, readFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
export async function browser() {
  const origin = process.env.LUKAS_BROWSER_ORIGIN ?? 'http://localhost:4173'
  const profile = await mkdtemp(join(tmpdir(), 'lukas-browser-'))
  const executable = process.env.LUKAS_BROWSER_BIN ?? 'chromium'
  const browserProcess = spawn(executable, ['--headless=new', '--no-sandbox', '--disable-dev-shm-usage', '--remote-debugging-port=0', `--user-data-dir=${profile}`, `${origin}/accounts`], { stdio: 'ignore' })
  let ws
  const sleep = ms => new Promise(r => setTimeout(r, ms))
  try {
    for (let i=0; i<60; i++) {
      try { const port=(await readFile(join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0]; const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json(); const page=targets.find(t=>t.type==='page' && t.url===`${origin}/accounts`); if(page) {ws=new WebSocket(page.webSocketDebuggerUrl); break} } catch {}
      await sleep(250)
    }
    if(!ws) throw new Error('Chromium did not start')
    await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject})
    let id=0; const pending=new Map()
    ws.onmessage=e=>{const m=JSON.parse(e.data);if(!m.id)return;const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(new Error(m.error.message)):p.resolve(m.result)}
    const command=(method,params={})=>new Promise((resolve,reject)=>{const n=++id;pending.set(n,{resolve,reject});ws.send(JSON.stringify({id:n,method,params}))})
    const evaluate=async expression=>{const r=await command('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails)throw new Error(JSON.stringify(r.exceptionDetails));return r.result.value}
    const waitFor=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await sleep(100)}throw new Error(`Timed out: ${expression}; body: ${await evaluate('document.body.innerText')}`)}
    const set=async(selector,value)=>{await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw new Error('Missing control');e.value=${JSON.stringify(value)};e.dispatchEvent(new Event(e.tagName==='SELECT'?'change':'input',{bubbles:true}))})()`);await sleep(30)}
    const click=async selector=>{await waitFor(`Boolean(document.querySelector(${JSON.stringify(selector)})) && !document.querySelector(${JSON.stringify(selector)}).disabled`);await evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);await sleep(30)}
    return { origin,command,evaluate,waitFor,set,click,close:async()=>{ws.close();browserProcess.kill();await sleep(250);await rm(profile,{recursive:true,force:true})} }
  } catch(error) {browserProcess.kill();await rm(profile,{recursive:true,force:true});throw error}
}
