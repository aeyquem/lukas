import assert from 'node:assert/strict'
import { browser } from './chromium.mjs'
const b=await browser()
try{
 await b.command('Page.navigate',{url:`${b.origin}/transactions`})
 await b.waitFor(`document.querySelector('#transaction-wallet')?.options.length>1`)
 const checks=[]
 for(const kind of ['income','expense','refund','transfer']){
  await b.set('#transaction-kind',kind)
  await b.set(kind==='transfer'?'#source-amount':'#transaction-amount','0')
  await b.click('button[type="submit"]')
  await b.waitFor(`Boolean(document.querySelector('[aria-invalid="true"]'))`)
  await b.command('Page.bringToFront')
  await b.evaluate(`document.querySelector('nav a').focus()`)
  const aria=await b.evaluate(`(()=>{const form=document.querySelector('form');const controls=[...form.querySelectorAll('input,select,textarea,button')].filter(e=>e.type!=='hidden'&&e.getClientRects().length);const referencedText=(e,attribute)=>((e.getAttribute(attribute)||'').trim().split(/\\s+/).filter(Boolean).map(id=>document.getElementById(id)?.textContent.trim()||'').filter(Boolean).join(' ')).trim();const accessibleName=e=>(e.getAttribute('aria-label')||'').trim()||referencedText(e,'aria-labelledby')||[...(e.labels||[])].map(label=>label.textContent.trim()).filter(Boolean).join(' ').trim()||e.closest('label')?.textContent.trim()||((e.tagName==='BUTTON'||e.type==='submit'||e.type==='button')?(e.innerText||e.value||'').trim():'');const ids=[...document.querySelectorAll('[id]')].map(e=>e.id);const duplicates=ids.filter((id,i)=>ids.indexOf(id)!==i);const brokenReferences=[...document.querySelectorAll('[aria-describedby],[aria-labelledby]')].flatMap(e=>['aria-describedby','aria-labelledby'].flatMap(a=>(e.getAttribute(a)||'').trim().split(/\\s+/).filter(Boolean).filter(id=>!document.getElementById(id)).map(id=>({attribute:a,id}))));const main=document.querySelector('main');const invalid=controls.filter(e=>e.getAttribute('aria-invalid')==='true');const invalidWithoutError=invalid.filter(e=>(e.getAttribute('aria-describedby')||'').trim().split(/\\s+/).filter(Boolean).map(id=>document.getElementById(id)).filter(Boolean).every(node=>!node.classList.contains('field-error')||!node.textContent.trim())).map(e=>e.id||e.tagName);return {controlCount:controls.length,invalidCount:invalid.length,unnamed:controls.filter(e=>!accessibleName(e)).map(e=>({tag:e.tagName,id:e.id,type:e.type})),duplicateIds:[...new Set(duplicates)],brokenReferences,invalidWithoutError,mainHasName:Boolean(main&&((main.getAttribute('aria-label')||'').trim()||referencedText(main,'aria-labelledby')))}})()`)
  assert.deepEqual(aria.unnamed,[],`${kind} controls need an accessible name`)
  assert.deepEqual(aria.duplicateIds,[],`${kind} route has duplicate IDs`)
  assert.deepEqual(aria.brokenReferences,[],`${kind} route has a broken ARIA reference`)
  assert.ok(aria.invalidCount>0,`${kind} invalid submission should expose an aria-invalid control`)
  assert.deepEqual(aria.invalidWithoutError,[],`${kind} invalid controls need a non-empty error description`)
  assert.equal(aria.mainHasName,true,`${kind} route main landmark needs a name`)
  const required=await b.evaluate(`(()=>Array.from(document.querySelectorAll('form input,form select,form textarea,form button')).filter(e=>!e.disabled&&e.type!=='hidden'&&e.getClientRects().length).map((e,index)=>{e.dataset.keyboardAuditIndex=String(index);return {index:String(index),id:e.id,tag:e.tagName,type:e.type}}))()`)
  const reached=new Set()
  for(let i=0;i<90;i++){
   await b.command('Input.dispatchKeyEvent',{type:'keyDown',key:'Tab',code:'Tab',windowsVirtualKeyCode:9})
   await b.command('Input.dispatchKeyEvent',{type:'keyUp',key:'Tab',code:'Tab',windowsVirtualKeyCode:9})
   reached.add(await b.evaluate('document.activeElement?.dataset.keyboardAuditIndex??null'))
   if(required.every(e=>reached.has(e.index)))break
  }
  assert.deepEqual(required.filter(e=>!reached.has(e.index)),[],`Unreachable ${kind} controls`)
  checks.push(`${kind}: ${required.length} controls reached using Tab; ${aria.controlCount} controls passed ARIA name/reference checks; ${aria.invalidCount} invalid controls referenced non-empty errors`)
 }
 console.log(JSON.stringify({result:'PASS',checks}))
}finally{await b.close()}
