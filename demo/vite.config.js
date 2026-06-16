import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs'
import path from 'path'
import { exec } from 'child_process'

let deployedFiles = {}

function deployMiddleware() {
  return {
    name: 'deploy-middleware',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        // Intercept API deploy endpoint
        if (req.url === '/api/deploy' && req.method === 'POST') {
          let body = ''
          req.on('data', chunk => {
            body += chunk.toString()
          })
          req.on('end', () => {
            try {
              const data = JSON.parse(body)
              const { files } = data
              if (!files) {
                res.statusCode = 400
                res.setHeader('Content-Type', 'application/json')
                res.end(JSON.stringify({ error: 'Missing files in payload' }))
                return
              }

              // Save to memory
              deployedFiles = files

              const host = req.headers.host || 'localhost:5173'
              const url = `http://${host}/`

              res.statusCode = 200
              res.setHeader('Content-Type', 'application/json')
              res.end(JSON.stringify({ success: true, url }))
            } catch (err) {
              console.error('Payload processing error:', err)
              res.statusCode = 500
              res.setHeader('Content-Type', 'application/json')
              res.end(JSON.stringify({ error: `Server error: ${err.message}` }))
            }
          })
          return
        }

        // Intercept PWA routes at the root level (/)
        let pathname = req.url.split('?')[0]
        if (
          pathname === '/manifest.json' ||
          pathname === '/sw.js' ||
          pathname.startsWith('/favicon/')
        ) {
          let relPath = pathname.substring(1) // Remove leading slash
          if (pathname === '/manifest.json') {
            relPath = 'favicon/manifest.json'
          }

          // Check if file is in memory
          const base64Data = deployedFiles[relPath]
          if (base64Data) {
            let contentType = 'text/plain'
            if (relPath.endsWith('.js')) contentType = 'application/javascript; charset=utf-8'
            else if (relPath.endsWith('.json')) contentType = 'application/json; charset=utf-8'
            else if (relPath.endsWith('.ico')) contentType = 'image/x-icon'
            else if (relPath.endsWith('.png')) contentType = 'image/png'

            res.statusCode = 200
            res.setHeader('Content-Type', contentType)
            res.end(Buffer.from(base64Data, 'base64'))
            return
          } else {
            // Serve default fallbacks if not customized/generated yet
            if (pathname === '/manifest.json') {
              res.statusCode = 200
              res.setHeader('Content-Type', 'application/json; charset=utf-8')
              res.end(JSON.stringify({
                name: 'Favicon Generator',
                short_name: 'Favicon',
                start_url: '/',
                display: 'standalone',
                icons: []
              }))
              return
            } else if (pathname === '/sw.js') {
              res.statusCode = 200
              res.setHeader('Content-Type', 'application/javascript; charset=utf-8')
              res.end(`self.addEventListener('install', (e) => { self.skipWaiting(); });\nself.addEventListener('activate', (e) => { e.waitUntil(clients.claim()); });\nself.addEventListener('fetch', (e) => { e.respondWith(fetch(e.request)); });`)
              return
            }
          }
        }

        next()
      })
    }
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), deployMiddleware()],
  server: {
    fs: {
      allow: ['..']
    }
  }
})

