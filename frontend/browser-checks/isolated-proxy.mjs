// Test-only host proxy: UI from the dev proxy, API from an isolated database.
import http from 'node:http'
const server = http.createServer((req, res) => {
  const api = req.url.startsWith('/api/') || req.url.startsWith('/health')
  const upstream = http.request({ hostname: '127.0.0.1', port: api ? 4175 : 4173, path: req.url, method: req.method, headers: { ...req.headers, host: api ? 'localhost:4174' : 'localhost:4173' } }, response => {
    res.writeHead(response.statusCode, response.headers)
    response.pipe(res)
  })
  upstream.on('error', error => { res.writeHead(502); res.end(error.message) })
  req.pipe(upstream)
})
server.listen(4174, '127.0.0.1', () => console.log('Isolated browser proxy listening at http://localhost:4174'))
