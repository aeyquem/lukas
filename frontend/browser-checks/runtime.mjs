import assert from 'node:assert/strict'
import http from 'node:http'
const mode=process.argv[2]??'healthy'
const origin='http://localhost:4174'
const ready=await fetch(`${origin}/health/ready`,{signal:AbortSignal.timeout(15000)})
assert.equal(ready.status,mode==='outage'?503:200)
assert.equal((await fetch(`${origin}/health/live`)).status,200)
if(mode==='healthy'){
 for(const headers of [{},{Origin:'http://evil.example'}]){
  const response=await fetch('http://localhost:4175/api/accounts',{method:'POST',headers,body:'{}'})
  assert.equal(response.status,403)
 }
 let rejected=false
 try{await new Promise((resolve,reject)=>{const r=http.get('http://localhost:4173/',{headers:{Host:'evil.example'}},response=>{response.resume();resolve(response.statusCode)});r.on('error',reject);r.setTimeout(5000,()=>r.destroy(new Error('Host probe timeout')))})}catch{rejected=true}
 assert.equal(rejected,true)
}
console.log(JSON.stringify({result:'PASS',mode,readiness:ready.status,liveness:200,checks:mode==='healthy'?['exact Origin required for mutations','invalid proxy Host rejected']:['isolated database outage reflected in readiness']}))
