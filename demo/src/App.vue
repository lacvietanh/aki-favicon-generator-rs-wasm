<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import init, { generate_favicon_set, FaviconOptions } from '../../pkg/aki_favicon_generator.js'
import { zipSync } from 'fflate'

// ── State ─────────────────────────────────────────────────────────────────────
const isProcessing = ref(false)
const errorMsg = ref('')
const htmlSnippet = ref('')
const copied = ref(false)
const resultUrl = ref(null)

const appName = ref('My App')
const shortName = ref('App')
const selectedFileName = ref('')
const selectedFileSize = ref('')

const themeColor = ref('—')
const backgroundColor = ref('—')

const uiTheme = ref('light')
const fileInputRef = ref(null)
const deferredPrompt = ref(null)
const showInstallBtn = ref(false)



// ── PWA install ───────────────────────────────────────────────────────────────
window.addEventListener('beforeinstallprompt', (e) => {
  e.preventDefault()
  deferredPrompt.value = e
  showInstallBtn.value = true
})

const installPwa = async () => {
  if (!deferredPrompt.value) return
  deferredPrompt.value.prompt()
  const { outcome } = await deferredPrompt.value.userChoice
  console.log('User response to install prompt:', outcome)
  deferredPrompt.value = null
  showInstallBtn.value = false
}

window.addEventListener('appinstalled', () => {
  console.log('PWA was installed successfully')
  showInstallBtn.value = false
})

const refreshPwaManifest = () => {
  let link = document.querySelector('link[rel="manifest"]')
  if (!link) { link = document.createElement('link'); link.rel = 'manifest'; document.head.appendChild(link) }
  link.href = `/manifest.json?v=${Date.now()}`

  let meta = document.querySelector('meta[name="theme-color"]')
  if (!meta) { meta = document.createElement('meta'); meta.name = 'theme-color'; document.head.appendChild(meta) }
  meta.content = themeColor.value

  let appleLink = document.querySelector('link[rel="apple-touch-icon"]')
  if (!appleLink) { appleLink = document.createElement('link'); appleLink.rel = 'apple-touch-icon'; document.head.appendChild(appleLink) }
  appleLink.href = `/favicon/apple-touch-icon.png?v=${Date.now()}`

  const svgLink = document.querySelector('link[type="image/svg+xml"]')
  if (svgLink) svgLink.remove()

  let icoLink = document.querySelector('link[rel="icon"]:not([type="image/svg+xml"])')
  if (!icoLink) { icoLink = document.createElement('link'); icoLink.rel = 'icon'; document.head.appendChild(icoLink) }
  icoLink.href = `/favicon/favicon.ico?v=${Date.now()}`
}

onMounted(() => {
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/sw.js')
      .then(reg => console.log('Service Worker registered', reg))
      .catch(err => console.error('Service Worker registration failed', err))
  }
})

// ── Preview URLs ──────────────────────────────────────────────────────────────
const previewUrls = ref({ faviconIco: null, icon192: null, icon512Maskable: null, appleTouch: null })

const currentYear = new Date().getFullYear()

const sizeAndSpeedFacts = computed(() => {
  const wasmBytes = 361911
  const wasmKb = (wasmBytes / 1024).toFixed(1)
  return [
    { label: 'WASM Size', value: `${wasmKb} KB` },
    { label: 'Target Bundle', value: '≤ 2 MB' },
    { label: 'Processing', value: '< 1 second' },
    { label: 'Server Cost', value: 'Zero' },
  ]
})

const outputArtifacts = [
  { name: 'favicon.ico',           size: '16×16 + 32×32', role: 'Browser tab + bookmarks',            purpose: 'Transparent canvas, rounded-square normalized, multi-res ICO.' },
  { name: 'icon-192.png',          size: '192×192',        role: 'PWA fallback icon',                  purpose: 'purpose: "any" — broadly compatible for PWA manifest.' },
  { name: 'icon-512-maskable.png', size: '512×512',        role: 'Android adaptive icon + splash',     purpose: 'purpose: "maskable" — full-bleed opaque bg, 80% safe zone.' },
  { name: 'apple-touch-icon.png',  size: '180×180',        role: 'iOS home screen icon',               purpose: 'Edge-to-edge opaque bg avoids iOS black-fill on transparent images.' },
  { name: 'manifest.json',         size: 'JSON',           role: 'PWA metadata',                       purpose: 'Declares theme_color, background_color, icon purposes.' },
]

const mvpExclusions = [
  { item: 'SVG auto-tracing',          reason: 'Heavy vector deps, inflates bundle.' },
  { item: 'Monochrome badge',          reason: 'Not required for core favicon workflows.' },
  { item: 'Dark-mode icon variants',   reason: 'Needs separate source assets.' },
  { item: 'Open Graph image',          reason: 'Different domain — social sharing, not favicon.' },
  { item: 'Windows legacy tiles',      reason: 'Low modern value, out of MVP scope.' },
]

const architectureSteps = [
  'Decode input image bytes (PNG/JPEG) in WASM.',
  'Auto-detect theme/background colors from pixel histogram.',
  'Apply safe-zone fitting and solid/gradient background where required.',
  'Generate multi-size outputs with Lanczos3 quality inside WASM.',
  'Assemble ZIP in JS using fflate with manifest + HTML snippet.',
]

// ── Helpers ───────────────────────────────────────────────────────────────────
const closeCopiedToast = () => setTimeout(() => { copied.value = false }, 1200)

const bytesToReadable = (bytes) => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

const revokePreviewUrls = () => {
  Object.values(previewUrls.value).forEach(u => { if (u) URL.revokeObjectURL(u) })
  previewUrls.value = { faviconIco: null, icon192: null, icon512Maskable: null, appleTouch: null }
}

const revokeZipUrl = () => { if (resultUrl.value) { URL.revokeObjectURL(resultUrl.value); resultUrl.value = null } }

const buildManifest = (result) => ({
  name: appName.value,
  short_name: shortName.value,
  start_url: '/',
  display: 'standalone',
  theme_color: result.theme_color,
  background_color: result.background_color,
  icons: [
    { src: '/favicon/icon-192.png', sizes: '192x192', type: 'image/png', purpose: 'any' },
    { src: '/favicon/icon-512-maskable.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
  ],
})

const buildSnippet = (theme) => `<link rel="icon" href="/favicon/favicon.ico" sizes="32x32">
<link rel="apple-touch-icon" sizes="180x180" href="/favicon/apple-touch-icon.png">
<link rel="icon" type="image/png" sizes="192x192" href="/favicon/icon-192.png">
<link rel="manifest" href="/favicon/manifest.json">
<meta name="theme-color" content="${theme}">`

const updatePreviewUrls = (result) => {
  revokePreviewUrls()
  previewUrls.value.faviconIco       = URL.createObjectURL(new Blob([result.favicon_ico],          { type: 'image/x-icon' }))
  previewUrls.value.icon192          = URL.createObjectURL(new Blob([result.icon_192],              { type: 'image/png' }))
  previewUrls.value.icon512Maskable  = URL.createObjectURL(new Blob([result.icon_512_maskable],    { type: 'image/png' }))
  previewUrls.value.appleTouch       = URL.createObjectURL(new Blob([result.apple_touch_icon],     { type: 'image/png' }))
}

const createZip = (result, manifest, snippet) => zipSync({
  'favicon/favicon.ico':            result.favicon_ico,
  'favicon/icon-192.png':           result.icon_192,
  'favicon/icon-512-maskable.png':  result.icon_512_maskable,
  'favicon/apple-touch-icon.png':   result.apple_touch_icon,
  'favicon/manifest.json':          new TextEncoder().encode(JSON.stringify(manifest, null, 2)),
  'html-snippet.txt':               new TextEncoder().encode(snippet),
})

const lastResult = ref(null)

const deployState = ref({ status: 'idle', url: '', error: '' })

const wallpaperGradient = ref('linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%)')

const randomizeWallpaper = () => {
  const angle = Math.floor(Math.random() * 360)
  const h1 = Math.floor(Math.random() * 360)
  const h2 = (h1 + 40 + Math.floor(Math.random() * 120)) % 360
  const mode = Math.random()
  let s1, l1, s2, l2
  if (mode < 0.33) { s1 = 30 + Math.floor(Math.random() * 60); l1 = 5 + Math.floor(Math.random() * 20); s2 = 30 + Math.floor(Math.random() * 60); l2 = 5 + Math.floor(Math.random() * 15) }
  else if (mode < 0.66) { s1 = 20 + Math.floor(Math.random() * 60); l1 = 75 + Math.floor(Math.random() * 20); s2 = 20 + Math.floor(Math.random() * 60); l2 = 70 + Math.floor(Math.random() * 20) }
  else { s1 = 60 + Math.floor(Math.random() * 40); l1 = 35 + Math.floor(Math.random() * 25); s2 = 60 + Math.floor(Math.random() * 40); l2 = 30 + Math.floor(Math.random() * 25) }
  wallpaperGradient.value = `linear-gradient(${angle}deg, hsl(${h1}, ${s1}%, ${l1}%) 0%, hsl(${h2}, ${s2}%, ${l2}%) 100%)`
}

const uint8ToBase64 = (arr) => { let b = ''; for (let i = 0; i < arr.byteLength; i++) b += String.fromCharCode(arr[i]); return btoa(b) }
const stringToBase64 = (str) => uint8ToBase64(new TextEncoder().encode(str))

const isColorDark = (hex) => {
  if (!hex || hex === '—' || hex[0] !== '#') return true
  const c = hex.substring(1); const rgb = parseInt(c, 16)
  const r = (rgb >> 16) & 0xff; const g = (rgb >> 8) & 0xff; const b = (rgb >> 0) & 0xff
  return 0.2126 * r + 0.7152 * g + 0.0722 * b < 128
}

const deployLivePwa = async () => {
  if (!lastResult.value) return
  deployState.value = { status: 'deploying', url: '', error: '' }
  try {
    const result = lastResult.value
    const manifest = buildManifest(result)
    const manifestStr = JSON.stringify(manifest, null, 2)
    const serviceWorkerJs = `self.addEventListener('install', (e) => { self.skipWaiting(); });
self.addEventListener('activate', (e) => { e.waitUntil(clients.claim()); });
self.addEventListener('fetch', (e) => { e.respondWith(fetch(e.request)); });`
    const payload = {
      files: {
        'favicon/favicon.ico':            uint8ToBase64(result.favicon_ico),
        'favicon/icon-192.png':           uint8ToBase64(result.icon_192),
        'favicon/icon-512-maskable.png':  uint8ToBase64(result.icon_512_maskable),
        'favicon/apple-touch-icon.png':   uint8ToBase64(result.apple_touch_icon),
        'favicon/manifest.json':          stringToBase64(manifestStr),
        'sw.js':                          stringToBase64(serviceWorkerJs),
      }
    }
    const response = await fetch('/api/deploy', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) })
    const data = await response.json()
    if (response.ok && data.success) { deployState.value.status = 'success'; deployState.value.url = data.url; refreshPwaManifest() }
    else { deployState.value.status = 'error'; deployState.value.error = data.error || 'Unknown deployment failure' }
  } catch (err) { deployState.value.status = 'error'; deployState.value.error = String(err) }
}

const processFile = async (file) => {
  if (!file) return
  errorMsg.value = ''; copied.value = false; isProcessing.value = true
  selectedFileName.value = file.name; selectedFileSize.value = bytesToReadable(file.size)
  deployState.value = { status: 'idle', url: '', error: '' }
  try {
    const imageBytes = new Uint8Array(await file.arrayBuffer())
    await init()
    const options = new FaviconOptions()
    const result = generate_favicon_set(imageBytes, options)
    lastResult.value = result
    themeColor.value = result.theme_color
    backgroundColor.value = result.background_color
    const snippet = buildSnippet(result.theme_color)
    const manifest = buildManifest(result)
    htmlSnippet.value = snippet
    updatePreviewUrls(result)
    const zip = createZip(result, manifest, snippet)
    revokeZipUrl()
    resultUrl.value = URL.createObjectURL(new Blob([zip], { type: 'application/zip' }))
  } catch (err) { errorMsg.value = String(err) }
  finally { isProcessing.value = false }
}

const onFileInputChange = async (e) => { await processFile(e.target.files?.[0]) }
const onDrop = async (e) => { await processFile(e.dataTransfer?.files?.[0]) }
const openFilePicker = () => { fileInputRef.value?.click() }

const copySnippet = async () => {
  if (!htmlSnippet.value) return
  await navigator.clipboard.writeText(htmlSnippet.value)
  copied.value = true; closeCopiedToast()
}

const toggleTheme = () => { uiTheme.value = uiTheme.value === 'light' ? 'dark' : 'light' }

onBeforeUnmount(() => { revokePreviewUrls(); revokeZipUrl() })
</script>

<template>
  <main class="page" :data-theme="uiTheme">

    <!-- ── Page Header ─────────────────────────────────────────────────────── -->
    <header class="page-header">
      <div class="header-brand">
        <h1>Aki Favicon Generator</h1>
        <p>One image → 4 production-ready icons + manifest. 100% client-side, zero server cost.</p>
      </div>
      <div class="header-meta">
        <div class="stat-pills">
          <span v-for="f in sizeAndSpeedFacts" :key="f.label" class="stat-pill">
            <em>{{ f.label }}</em>
            <strong>{{ f.value }}</strong>
          </span>
        </div>
        <button type="button" class="btn-icon" @click="toggleTheme" title="Toggle theme">
          {{ uiTheme === 'light' ? '🌙' : '☀️' }}
        </button>
      </div>
    </header>

    <!-- ── Main Workspace (3-column) ──────────────────────────────────────── -->
    <div class="workspace">

      <!-- Left: Controls -->
      <aside class="col-controls panel">
        <h2>Configuration</h2>
        <p class="helper">Upload once. All 4 icons generated in under 1 second.</p>

        <div class="form-grid">
          <label>
            App Name
            <input v-model="appName" type="text" placeholder="My App" />
          </label>
          <label>
            Short Name <small class="label-hint">max 12 chars</small>
            <input v-model="shortName" type="text" maxlength="12" placeholder="App" />
          </label>
        </div>

        <div
          class="dropzone"
          role="button"
          tabindex="0"
          @click="openFilePicker"
          @keydown.enter.prevent="openFilePicker"
          @keydown.space.prevent="openFilePicker"
          @dragover.prevent
          @drop.prevent="onDrop"
        >
          <input ref="fileInputRef" class="hidden-input" type="file" accept="image/png, image/jpeg" :disabled="isProcessing" @change="onFileInputChange" />
          <span class="dropzone-icon">{{ isProcessing ? '⏳' : '📁' }}</span>
          <strong>{{ isProcessing ? 'Processing…' : 'Drop image or click to upload' }}</strong>
          <span>PNG or JPEG</span>
          <small v-if="selectedFileName">{{ selectedFileName }} <em>({{ selectedFileSize }})</em></small>
        </div>

        <p v-if="errorMsg" class="error">{{ errorMsg }}</p>

        <div class="action-stack">
          <a v-if="resultUrl" :href="resultUrl" download="favicon.zip" class="btn btn-primary">⬇ Download favicon.zip</a>
          <button v-else class="btn btn-primary" type="button" disabled>⬇ Download favicon.zip</button>

          <button
            v-if="lastResult"
            class="btn btn-secondary"
            type="button"
            :disabled="deployState.status === 'deploying'"
            @click="deployLivePwa"
          >
            {{ deployState.status === 'deploying' ? '⏳ Deploying...' : '🚀 Deploy & Test PWA Live' }}
          </button>
          <button v-else class="btn btn-secondary" type="button" disabled>🚀 Deploy & Test PWA Live</button>
        </div>

        <!-- Deploy status -->
        <div v-if="deployState.status !== 'idle'" class="deploy-panel">
          <div v-if="deployState.status === 'deploying'" class="deploy-loading">
            <span class="spinner" />
            <span>Deploying to local test environment...</span>
          </div>
          <div v-else-if="deployState.status === 'success'" class="deploy-success">
            <p><strong>✓ PWA ready to install!</strong></p>
            <button v-if="showInstallBtn" class="btn btn-primary" style="margin: 0.5rem 0;" type="button" @click="installPwa">Install Custom PWA</button>
            <p v-else class="hint-text">Click the install icon in the address bar, or scan QR on mobile:</p>
            <div class="qr-container">
              <img :src="`https://api.qrserver.com/v1/create-qr-code/?size=140x140&data=${encodeURIComponent(deployState.url)}`" alt="QR Code" class="qr-code" />
            </div>
          </div>
          <div v-else-if="deployState.status === 'error'" class="deploy-error">
            <p><strong>✗ Deployment failed</strong></p>
            <code class="error-detail">{{ deployState.error }}</code>
          </div>
        </div>
      </aside>

      <!-- Center: Icon Preview (flat gradient area, all icons visible) -->
      <div class="col-mockup panel">
        <div class="preview-header">
          <h2>Live Preview</h2>
          <button type="button" class="btn-ghost" @click="randomizeWallpaper">🎲 Wallpaper</button>
        </div>

        <!-- Gradient stage: all icons laid out flat, no phone shell -->
        <div class="preview-stage" :style="{ background: wallpaperGradient }">

          <!-- Android maskable: 3 shapes -->
          <div class="preview-group">
            <div class="preview-icon-row">
              <div class="preview-app-item">
                <div class="preview-icon shape-circle">
                  <img v-if="previewUrls.icon512Maskable" :src="previewUrls.icon512Maskable" alt="Circle" />
                  <div v-else class="icon-placeholder" />
                </div>
                <span class="preview-app-name">{{ shortName || 'App' }}</span>
                <span class="preview-platform-hint">Android Circle</span>
              </div>
              <div class="preview-app-item">
                <div class="preview-icon shape-squircle">
                  <img v-if="previewUrls.icon512Maskable" :src="previewUrls.icon512Maskable" alt="Squircle" />
                  <div v-else class="icon-placeholder" />
                </div>
                <span class="preview-app-name">{{ shortName || 'App' }}</span>
                <span class="preview-platform-hint">Android Squircle</span>
              </div>
              <div class="preview-app-item">
                <div class="preview-icon shape-rounded-square">
                  <img v-if="previewUrls.icon512Maskable" :src="previewUrls.icon512Maskable" alt="Rounded" />
                  <div v-else class="icon-placeholder" />
                </div>
                <span class="preview-app-name">{{ shortName || 'App' }}</span>
                <span class="preview-platform-hint">Android Rounded</span>
              </div>
              <!-- iOS -->
              <div class="preview-app-item">
                <div class="preview-icon shape-ios">
                  <img v-if="previewUrls.appleTouch" :src="previewUrls.appleTouch" alt="iOS" />
                  <div v-else class="icon-placeholder" />
                </div>
                <span class="preview-app-name">{{ shortName || 'App' }}</span>
                <span class="preview-platform-hint">iOS</span>
              </div>
              <!-- Web App / PWA any -->
              <div class="preview-app-item">
                <div class="preview-icon shape-plain">
                  <img v-if="previewUrls.icon192" :src="previewUrls.icon192" alt="Web App" />
                  <div v-else class="icon-placeholder" />
                </div>
                <span class="preview-app-name">{{ shortName || 'App' }}</span>
                <span class="preview-platform-hint">Web App</span>
              </div>
            </div>
          </div>

          <!-- Browser favicon strip at bottom of stage -->
          <div class="preview-browser-strip">
            <div class="preview-tab-pill">
              <!-- favicon: direct WASM output — no CSS border-radius override -->
              <img v-if="previewUrls.faviconIco" :src="previewUrls.faviconIco" alt="favicon" class="preview-favicon" />
              <div v-else class="favicon-placeholder" />
              <span class="preview-tab-name">{{ shortName || 'App' }}</span>
              <span class="preview-tab-platform">Browser Tab</span>
            </div>
          </div>
        </div>

        <!-- Source file captions -->
        <div class="preview-captions">
          <span>Android &amp; Web App → <code>icon-512-maskable.png</code></span>
          <span>iOS → <code>apple-touch-icon.png</code></span>
          <span>Browser → <code>favicon.ico</code> (WASM rounded-square)</span>
        </div>
      </div>

      <!-- Right: Info panel -->
      <div class="col-info">
        <!-- Color swatches -->
        <div class="panel info-card">
          <h3>Detected Colors</h3>
          <div class="swatch-list">
            <div class="swatch-item">
              <div class="swatch-dot" :style="{ background: themeColor !== '—' ? themeColor : 'var(--muted-border)' }" />
              <div>
                <span class="swatch-label">Theme Color</span>
                <code>{{ themeColor }}</code>
              </div>
            </div>
            <div class="swatch-item">
              <div class="swatch-dot" :style="{ background: backgroundColor !== '—' ? backgroundColor : 'var(--muted-border)' }" />
              <div>
                <span class="swatch-label">Background Color</span>
                <code>{{ backgroundColor }}</code>
              </div>
            </div>
          </div>
        </div>

        <!-- Artifacts list -->
        <div class="panel info-card">
          <h3>Output Artifacts</h3>
          <ul class="artifact-list">
            <li v-for="a in outputArtifacts" :key="a.name" class="artifact-item">
              <div class="artifact-header">
                <code class="artifact-name">{{ a.name }}</code>
                <span class="artifact-size">{{ a.size }}</span>
              </div>
              <span class="artifact-role">{{ a.role }}</span>
              <span class="artifact-purpose">{{ a.purpose }}</span>
            </li>
          </ul>
        </div>

        <!-- HTML Snippet -->
        <div class="panel info-card">
          <div class="snippet-header">
            <h3>HTML &lt;head&gt; Snippet</h3>
            <button type="button" class="copy-btn" @click="copySnippet">
              {{ copied ? '✓ Copied' : 'Copy' }}
            </button>
          </div>
          <pre class="snippet-pre">{{ htmlSnippet || 'Generate once to get the HTML snippet.' }}</pre>
        </div>
      </div>

    </div><!-- /workspace -->


    <!-- ── Knowledge Section (always expanded) ───────────────────────────────── -->
    <section class="knowledge-section">
      <h2 class="knowledge-title">Architecture &amp; Philosophy</h2>
      <div class="knowledge-grid">

        <div class="knowledge-card panel">
          <h3>Why only 5 files?</h3>
          <table class="knowledge-table">
            <thead><tr><th>Artifact</th><th>Role</th><th>Why</th></tr></thead>
            <tbody>
              <tr v-for="a in outputArtifacts" :key="a.name">
                <td><code>{{ a.name }}</code><small>{{ a.size }}</small></td>
                <td>{{ a.role }}</td>
                <td>{{ a.purpose }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="knowledge-card panel">
          <h3>MVP philosophy — what we excluded</h3>
          <ul class="excl-list">
            <li v-for="row in mvpExclusions" :key="row.item">
              <strong>{{ row.item }}</strong> — {{ row.reason }}
            </li>
          </ul>
        </div>

        <div class="knowledge-card panel">
          <h3>Architecture in 5 steps</h3>
          <ol class="step-list">
            <li v-for="step in architectureSteps" :key="step">{{ step }}</li>
          </ol>
          <p class="hint-text" style="margin-top: 0.6rem;">Safe zone rule: 80% content area with solid/gradient background for maskable and apple-touch outputs.</p>
        </div>

        <div class="knowledge-card panel">
          <h3>Performance &amp; bundle strategy</h3>
          <ul class="excl-list">
            <li>Rust WASM handles pixel math and encoding where JS would be slower.</li>
            <li>ZIP assembly and manifest stay in JS — no <code>serde_json</code> in WASM bundle.</li>
            <li><code>image</code> crate enables only PNG + JPEG features to minimize size.</li>
            <li>Release flags: <code>opt-level=z</code>, LTO, strip, wasm-opt post-pass.</li>
            <li>Main-thread execution is practical for typical input sizes (&lt; 5 MB).</li>
          </ul>
        </div>

      </div>
    </section>


    <footer class="footer">
      <p>© {{ currentYear }} Akitao · <a href="https://akitao.com" target="_blank" rel="noopener noreferrer">akitao.com</a> · <a href="mailto:admin@akitao.com">admin@akitao.com</a></p>
    </footer>

  </main>
</template>

<style scoped>
/* ── Tokens ─────────────────────────────────────────────────────────────────── */
.page {
  min-height: 100vh;
  padding: 1.2rem clamp(0.8rem, 2vw, 2rem) 2rem;
  background: var(--bg);
  color: var(--text);
  transition: background 0.2s, color 0.2s;
}

.page[data-theme='light'] {
  --bg: radial-gradient(circle at top, #e8efff 0%, #f7f8fc 38%, #f4f4f5 100%);
  --text: #111827;
  --text-soft: #4b5563;
  --panel-bg: rgba(255, 255, 255, 0.92);
  --panel-border: #dbe2ef;
  --panel-shadow: 0 12px 28px rgba(15, 23, 42, 0.08);
  --muted-bg: #f3f6ff;
  --muted-border: #d1d5db;
  --brand: #2563eb;
  --brand-2: #3b82f6;
  --danger: #b91c1c;
  --code-bg: #0f172a;
  --code-text: #cbd5e1;
  --tab-active-bg: #2563eb;
  --tab-active-text: #fff;
  --placeholder: #cbd5e1;
}

.page[data-theme='dark'] {
  --bg: radial-gradient(circle at top, #1f2937 0%, #111827 46%, #0b1020 100%);
  --text: #e5e7eb;
  --text-soft: #9ca3af;
  --panel-bg: rgba(17, 24, 39, 0.82);
  --panel-border: #374151;
  --panel-shadow: 0 12px 28px rgba(0, 0, 0, 0.35);
  --muted-bg: #1f2937;
  --muted-border: #4b5563;
  --brand: #60a5fa;
  --brand-2: #3b82f6;
  --danger: #f87171;
  --code-bg: #020617;
  --code-text: #cbd5e1;
  --tab-active-bg: #3b82f6;
  --tab-active-text: #fff;
  --placeholder: #374151;
}

/* ── Page Header ─────────────────────────────────────────────────────────────  */
.page-header {
  max-width: 1280px;
  margin: 0 auto 1.1rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}

.header-brand h1 {
  margin: 0;
  font-size: clamp(1.4rem, 2.2vw, 2rem);
  line-height: 1.2;
}

.header-brand p {
  margin: 0.3rem 0 0;
  color: var(--text-soft);
  font-size: 0.87rem;
}

.header-meta {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.stat-pills {
  display: flex;
  gap: 0.45rem;
  flex-wrap: wrap;
}

.stat-pill {
  display: flex;
  flex-direction: column;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 10px;
  padding: 0.35rem 0.65rem;
  font-size: 0.72rem;
  line-height: 1.3;
}

.stat-pill em { color: var(--text-soft); font-style: normal; }
.stat-pill strong { font-size: 0.82rem; }

.btn-icon {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 10px;
  padding: 0.45rem 0.7rem;
  font-size: 1.1rem;
  cursor: pointer;
  line-height: 1;
}

/* ── Workspace (3-col grid) ──────────────────────────────────────────────── */
.workspace {
  max-width: 1280px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: 290px 340px 1fr;
  gap: 1rem;
  align-items: start;
}

.panel {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  box-shadow: var(--panel-shadow);
  padding: 1rem;
}

/* ── Left: Controls ─────────────────────────────────────────────────────── */
.col-controls h2 { margin: 0; font-size: 1rem; }
.helper { margin: 0.35rem 0 0.75rem; color: var(--text-soft); font-size: 0.83rem; }

.form-grid { display: grid; gap: 0.6rem; }

label {
  display: flex;
  flex-direction: column;
  gap: 0.28rem;
  font-size: 0.8rem;
  font-weight: 700;
}

.label-hint { font-weight: 400; color: var(--text-soft); margin-left: 0.3rem; }

input {
  border: 1px solid var(--muted-border);
  border-radius: 9px;
  padding: 0.5rem 0.6rem;
  font-size: 0.9rem;
  background: var(--muted-bg);
  color: var(--text);
}

input:focus { outline: 2px solid var(--brand); outline-offset: 1px; }

.dropzone {
  margin-top: 0.75rem;
  border: 1.5px dashed var(--brand);
  border-radius: 12px;
  background: var(--muted-bg);
  padding: 0.85rem 0.75rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  cursor: pointer;
  text-align: center;
  transition: background 0.15s;
}

.dropzone:hover { background: var(--panel-border); }

.dropzone-icon { font-size: 1.5rem; }
.dropzone strong { font-size: 0.88rem; }
.dropzone span, .dropzone small { color: var(--text-soft); font-size: 0.78rem; }
.dropzone small em { font-style: normal; }

.hidden-input { display: none; }

.action-stack { display: flex; flex-direction: column; gap: 0.5rem; margin-top: 0.8rem; }

.btn {
  width: 100%;
  border: 0;
  border-radius: 10px;
  padding: 0.58rem 0.7rem;
  font-weight: 800;
  font-size: 0.87rem;
  text-align: center;
  text-decoration: none;
  cursor: pointer;
  display: block;
}

.btn-primary { background: linear-gradient(135deg, var(--brand) 0%, var(--brand-2) 100%); color: #fff; }
.btn-primary:disabled { opacity: 0.42; cursor: not-allowed; }

.btn-secondary {
  background: var(--muted-bg);
  border: 1px solid var(--muted-border);
  color: var(--text);
  transition: background 0.15s;
}
.btn-secondary:hover { background: var(--panel-border); }
.btn-secondary:disabled { opacity: 0.42; cursor: not-allowed; }

.error { color: var(--danger); font-size: 0.84rem; margin-top: 0.6rem; }

.deploy-panel { margin-top: 0.75rem; padding: 0.75rem; border-radius: 10px; background: var(--muted-bg); border: 1px solid var(--muted-border); font-size: 0.83rem; }
.deploy-loading { display: flex; align-items: center; gap: 0.5rem; color: var(--text-soft); }
.deploy-success p { margin: 0 0 0.35rem; }
.deploy-error { color: var(--danger); }
.hint-text { color: var(--text-soft); font-size: 0.79rem; margin: 0.3rem 0; }

.error-detail {
  display: block;
  font-size: 0.72rem;
  white-space: pre-wrap;
  margin-top: 0.35rem;
  background: rgba(239, 68, 68, 0.1);
  padding: 0.4rem;
  border-radius: 6px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
}

.qr-container { display: flex; justify-content: center; margin-top: 0.6rem; background: #fff; padding: 0.45rem; border-radius: 8px; width: fit-content; margin-left: auto; margin-right: auto; }
.qr-code { width: 140px; height: 140px; display: block; }

/* ── Center: Mockup ─────────────────────────────────────────────────────── */
.col-mockup { display: flex; flex-direction: column; gap: 0; }

.platform-tabs {
  display: flex;
  gap: 0.25rem;
  margin-bottom: 0.85rem;
  background: var(--muted-bg);
  border-radius: 12px;
  padding: 0.25rem;
  border: 1px solid var(--muted-border);
}

.platform-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
  padding: 0.45rem 0.5rem;
  border: 0;
  border-radius: 9px;
  background: transparent;
  color: var(--text-soft);
  font-size: 0.78rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s;
}

.platform-tab.active {
  background: var(--tab-active-bg);
  color: var(--tab-active-text);
  box-shadow: 0 2px 6px rgba(0,0,0,0.2);
}

.mockup-area { display: flex; flex-direction: column; align-items: center; gap: 0; }

.mockup-controls {
  align-self: flex-end;
  margin-bottom: 0.5rem;
}

.btn-ghost {
  border: 1px solid var(--muted-border);
  background: var(--muted-bg);
  color: var(--text);
  border-radius: 8px;
  padding: 0.3rem 0.65rem;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-ghost:hover { background: var(--panel-border); }

/* Phone shell */
.phone-shell {
  width: 100%;
  max-width: 300px;
  min-height: 300px;
  border-radius: 28px;
  border: 8px solid var(--panel-border);
  overflow: hidden;
  box-shadow: 0 20px 48px rgba(0,0,0,0.35), inset 0 1px 0 rgba(255,255,255,0.1);
  display: flex;
  flex-direction: column;
  transition: background 0.5s ease;
}

.phone-status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.55rem 1.1rem 0.15rem;
  color: rgba(255,255,255,0.88);
  font-size: 0.7rem;
  font-weight: 700;
}

.status-icons { display: flex; align-items: center; gap: 0.3rem; }

.phone-app-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem 0.5rem;
  padding: 1.2rem 0.9rem 1.5rem;
}

.ios-home-row {
  display: flex;
  justify-content: center;
  padding: 2rem 0.9rem 1.5rem;
}

.ios-home-row .phone-app-icon {
  width: 80px;
  height: 80px;
}

.ios-home-row .phone-app-label {
  font-size: 0.78rem;
}

.phone-app-item { display: flex; flex-direction: column; align-items: center; gap: 0.3rem; }

.phone-app-icon {
  width: 58px;
  height: 58px;
  overflow: hidden;
  display: grid;
  place-items: center;
  box-shadow: 0 6px 14px rgba(0,0,0,0.3);
  background: var(--placeholder);
  transition: transform 0.15s;
}

.phone-app-icon:hover { transform: scale(1.08); }
.phone-app-icon img { width: 100%; height: 100%; object-fit: cover; }

.icon-placeholder { width: 100%; height: 100%; background: var(--placeholder); }

.phone-app-label {
  font-size: 0.65rem;
  font-weight: 600;
  color: rgba(255,255,255,0.9);
  text-shadow: 0 1px 4px rgba(0,0,0,0.6);
  text-align: center;
  max-width: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shape-circle { border-radius: 50%; }
.shape-squircle { border-radius: 32%; }
.shape-rounded-square { border-radius: 18px; }
.shape-ios { border-radius: 22%; }
.shape-plain { border-radius: 10px; }

.mockup-caption {
  margin: 0.5rem 0 0;
  font-size: 0.73rem;
  color: var(--text-soft);
  text-align: center;
  line-height: 1.4;
}

.mockup-caption code { font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.72rem; }
.mockup-caption strong { color: var(--text); }

/* Browser mockup */
.browser-mockup-area { width: 100%; align-items: stretch; }

.browser-chrome {
  background: var(--muted-bg);
  border: 1px solid var(--muted-border);
  border-bottom: none;
  border-radius: 10px 10px 0 0;
  padding: 0.55rem 0.7rem 0;
}

.browser-traffic-lights { display: flex; gap: 0.35rem; margin-bottom: 0.45rem; }
.tl { width: 11px; height: 11px; border-radius: 50%; }
.tl-red { background: #ff5f57; }
.tl-yellow { background: #febc2e; }
.tl-green { background: #28c840; }

.browser-tab-strip { margin-bottom: 0.45rem; }

.browser-tab-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  background: var(--panel-bg);
  border: 1px solid var(--muted-border);
  border-bottom: none;
  border-radius: 8px 8px 0 0;
  padding: 0.35rem 0.55rem;
  font-size: 0.78rem;
  max-width: 180px;
}

/* favicon: direct WASM output — NO border-radius override here */
.browser-favicon { width: 16px; height: 16px; display: block; }
.favicon-placeholder { width: 16px; height: 16px; background: var(--placeholder); border-radius: 2px; }

.browser-tab-title {
  flex: 1;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.browser-tab-close { color: var(--text-soft); font-size: 0.9rem; cursor: default; }

.browser-address-bar {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  background: var(--panel-bg);
  border: 1px solid var(--muted-border);
  border-radius: 7px;
  padding: 0.3rem 0.6rem;
  font-size: 0.78rem;
  margin-bottom: 0.5rem;
  color: var(--text-soft);
}

.lock-icon { font-size: 0.72rem; }
.address-text { color: var(--text); font-size: 0.78rem; }

.browser-viewport {
  background: var(--panel-bg);
  border: 1px solid var(--muted-border);
  border-radius: 0 0 10px 10px;
  min-height: 160px;
  display: grid;
  place-items: center;
}

.browser-page-placeholder {
  color: var(--text-soft);
  font-size: 0.82rem;
  text-align: center;
  padding: 2rem;
}

/* ── Right: Info panel ──────────────────────────────────────────────────── */
.col-info { display: flex; flex-direction: column; gap: 1rem; }

.info-card h3 { margin: 0 0 0.65rem; font-size: 0.9rem; }

/* Color swatches */
.swatch-list { display: flex; flex-direction: column; gap: 0.5rem; }
.swatch-item { display: flex; align-items: center; gap: 0.55rem; }
.swatch-dot { width: 28px; height: 28px; border-radius: 8px; border: 1px solid rgba(148,163,184,0.35); flex-shrink: 0; }
.swatch-label { display: block; font-size: 0.72rem; color: var(--text-soft); margin-bottom: 0.1rem; }
.swatch-item code { font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.82rem; }

/* Artifacts */
.artifact-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0; }
.artifact-item { padding: 0.55rem 0; border-bottom: 1px solid var(--muted-border); }
.artifact-item:last-child { border-bottom: none; }
.artifact-header { display: flex; align-items: baseline; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.15rem; }
.artifact-name { font-size: 0.8rem; font-family: ui-monospace, Menlo, Consolas, monospace; color: var(--brand); }
.artifact-size { font-size: 0.72rem; color: var(--text-soft); flex-shrink: 0; }
.artifact-role { display: block; font-size: 0.78rem; font-weight: 700; color: var(--text); margin-bottom: 0.1rem; }
.artifact-purpose { display: block; font-size: 0.73rem; color: var(--text-soft); line-height: 1.4; }

/* Snippet */
.snippet-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
.snippet-header h3 { margin: 0; font-size: 0.9rem; }
.copy-btn { border: 1px solid var(--muted-border); background: var(--muted-bg); color: var(--text); border-radius: 7px; padding: 0.2rem 0.55rem; font-size: 0.73rem; cursor: pointer; }

.snippet-pre {
  margin: 0;
  padding: 0.65rem;
  font-size: 0.71rem;
  line-height: 1.5;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  background: var(--code-bg);
  color: var(--code-text);
  border-radius: 8px;
  overflow-x: auto;
  white-space: pre;
}

/* ── Knowledge Accordions ───────────────────────────────────────────────── */
.knowledge-section {
  max-width: 1280px;
  margin: 1.2rem auto 0;
}

.knowledge-title { font-size: 0.95rem; color: var(--text-soft); margin: 0 0 0.6rem; font-weight: 600; letter-spacing: 0.03em; text-transform: uppercase; }

.knowledge-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.75rem;
}

.knowledge-card h3 { margin: 0 0 0.7rem; font-size: 0.9rem; }

/* ── Preview stage ──────────────────────────────────────────────────────── */
.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.8rem;
}
.preview-header h2 { margin: 0; font-size: 1rem; }

.preview-stage {
  border-radius: 16px;
  padding: 1.6rem 1rem 1.2rem;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
  transition: background 0.5s ease;
}

.preview-icon-row {
  display: flex;
  gap: 0.7rem;
  justify-content: center;
  flex-wrap: wrap;
}

.preview-app-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.35rem;
}

.preview-icon {
  width: 52px;
  height: 52px;
  overflow: hidden;
  display: grid;
  place-items: center;
  box-shadow: 0 6px 16px rgba(0,0,0,0.35);
  transition: transform 0.15s;
  flex-shrink: 0;
}
.preview-icon:hover { transform: scale(1.1); }
.preview-icon img { width: 100%; height: 100%; object-fit: cover; }

.preview-app-name {
  font-size: 0.72rem;
  font-weight: 700;
  color: rgba(255,255,255,0.95);
  text-shadow: 0 1px 4px rgba(0,0,0,0.6);
  text-align: center;
  max-width: 72px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-platform-hint {
  font-size: 0.62rem;
  color: rgba(255,255,255,0.55);
  text-align: center;
}

/* Browser strip inside stage */
.preview-browser-strip {
  display: flex;
  justify-content: center;
}

.preview-tab-pill {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: rgba(255,255,255,0.15);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255,255,255,0.2);
  border-radius: 8px;
  padding: 0.4rem 0.75rem;
}

/* favicon: NO border-radius — direct WASM output */
.preview-favicon { width: 16px; height: 16px; display: block; }
.favicon-placeholder { width: 16px; height: 16px; background: rgba(255,255,255,0.3); border-radius: 2px; }

.preview-tab-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: rgba(255,255,255,0.9);
}

.preview-tab-platform {
  font-size: 0.68rem;
  color: rgba(255,255,255,0.5);
  margin-left: 0.25rem;
}

.preview-captions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem 1rem;
  margin-top: 0.65rem;
  font-size: 0.72rem;
  color: var(--text-soft);
}
.preview-captions code { font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.7rem; }

/* Icon placeholder (before image loaded) */
.icon-placeholder { width: 100%; height: 100%; background: rgba(255,255,255,0.12); }


.knowledge-table { width: 100%; border-collapse: collapse; font-size: 0.78rem; }
.knowledge-table th, .knowledge-table td { border-bottom: 1px solid var(--muted-border); padding: 0.4rem 0.3rem; text-align: left; vertical-align: top; line-height: 1.4; }
.knowledge-table th { color: var(--text-soft); font-size: 0.72rem; font-weight: 800; }
.knowledge-table td code { font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.72rem; display: block; }
.knowledge-table td small { color: var(--text-soft); font-size: 0.7rem; display: block; }

.excl-list, .step-list { margin: 0; padding-left: 1rem; display: flex; flex-direction: column; gap: 0.4rem; }
.excl-list li, .step-list li { font-size: 0.82rem; color: var(--text-soft); line-height: 1.4; }
.excl-list strong { color: var(--text); }
.excl-list li code { font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.78rem; }

/* ── Footer ─────────────────────────────────────────────────────────────── */
.footer { max-width: 1280px; margin: 1.2rem auto 0; text-align: center; color: var(--text-soft); font-size: 0.8rem; }
.footer a { color: inherit; text-decoration: none; border-bottom: 1px dashed currentColor; }

/* ── Spinner ─────────────────────────────────────────────────────────────  */
.spinner {
  width: 0.9rem; height: 0.9rem;
  border: 2px solid var(--text-soft);
  border-top-color: transparent;
  border-radius: 50%;
  display: inline-block;
  animation: spin 0.8s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* ── Responsive ─────────────────────────────────────────────────────────── */

/* Tablet: 2-col controls + mockup, info panel full-width below */
@media (max-width: 1060px) {
  .workspace {
    grid-template-columns: 1fr 1fr;
  }
  .col-info {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }
  .col-info .info-card { margin: 0; }
  .knowledge-grid { grid-template-columns: 1fr; }
}

/* Mobile: 1-col stack */
@media (max-width: 640px) {
  .workspace {
    grid-template-columns: 1fr;
  }
  .col-info {
    grid-template-columns: 1fr;
  }
  .header-meta {
    width: 100%;
    justify-content: space-between;
  }
  .stat-pills { flex-wrap: wrap; }
  .stat-pill { font-size: 0.68rem; }
  .knowledge-grid { grid-template-columns: 1fr; }
  .preview-icon-row { gap: 1rem; }
  .preview-icon { width: 56px; height: 56px; }
}
</style>
