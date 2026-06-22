# CRITICAL REFERENCE: FULL PWA MASTER BLUEPRINT
# WARNING: THIS IS A GLOBAL REFERENCE MANUAL CONTAINING FULL SPECIFICATIONS FOR PRODUCTION PWAS.
# DO NOT CROP OR ALIGN THIS TO THE GENERATOR'S MINIMAL MVP SPECIFICATIONS.
# IT IS INTENDED AS AN OVERALL WORKSPACE REFERENCE FOR WORKBOX, FIREBASE, FCM, AND OS APIS.

---

# MASTER BLUEPRINT: ULTIMATE FIREBASE PWA
# Context: Production-Grade, Native-Replacement, Cross-Platform (iOS/Android/Desktop)
# Stack: Firebase (Hosting, Firestore, Auth, FCM, Storage, Functions) + Workbox

---

## PART 0: OFFICIAL PWA GUIDANCE (2025)

### 0.1. MDN definition & best practices
* MDN describes a PWA as “an app that's built using web platform technologies, but that provides a user experience like that of a platform-specific app,” so the goal is installability, offline resilience, and device integration from a single codebase. ([MDN Progressive web apps](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps))
* The companion [MDN Best practices](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Best_practices) guide stresses cross-browser/device testing, progressive enhancement, custom offline handling, deeplinks, and OS integration (Notifications API, file_handlers, share_target, badges) to keep PWAs indistinguishable from native apps.

### 0.2. web.dev Core checklist (What makes a good PWA?)
Follow the five pillars from [web.dev’s PWA checklist](https://web.dev/articles/pwa-checklist#core) (last updated 2024-09-19):
1. **Starts fast, stays fast:** Aim for Core Web Vitals performance using Lighthouse, PageSpeed Insights, and Chrome UX Report so the first interaction and every scroll/animation feels instant.
2. **Works in any browser:** Design via progressive enhancement and feature detection so the core experience degrades gracefully across browsers and release channels.
3. **Responsive to any screen size:** Build layouts that rearrange content for a tiny handset up to a large desktop, keeping every task reachable and legible.
4. **Provides a custom offline page:** Install-time caching of a friendly offline fallback (see [Create an offline fallback page](https://web.dev/articles/offline-fallback-page)) keeps users in your brand instead of the browser’s error screen.
5. **Is installable:** Meet the [install criteria](https://web.dev/articles/install-criteria) by serving your app over HTTPS, registering a service worker that controls the `start_url`, and publishing a manifest with name/short_name, display/background colors, purpose-aware icons, and a valid scope.

### 0.3. web.dev Optimal checklist
For a truly platform-class experience, keep targeting the optimal checklist from the same web.dev article:
* **Provides an offline experience:** Complement cache-first routing with IndexedDB data stores, background sync, and deferred updates so users stay productive with or without a network.
* **Fully accessible:** Follow WCAG patterns, audit with Lighthouse, axe, and Accessibility Insights, and rely on semantic elements to satisfy every ability.
* **Discoverable in search:** Give each view a canonical URL, descriptive titles/descriptions, and submit via Search Console or Lighthouse SEO audits so organic users land in the correct context.
* **Works with any input type:** Support touch, mouse, pointer, stylus, keyboard, and voice by honoring Pointer Events, proper hit targets, and input-agnostic interactions.
* **Provides context for permission requests:** Trigger prompts (notifications, geolocation, etc.) only after explaining why the feature matters, as outlined in [Permission UX](https://web.dev/articles/push-notifications-permissions-ux).
* **Follows healthy code practices:** Keep dependencies updated, avoid deprecated APIs (no `document.write`, prefer passive listeners), and run cross-browser tests/lints before shipping.

### 0.4. Key APIs plus OS hooks
MDN’s reference pages catalog the APIs to achieve these goals:
* **Manifest members** (`name`, `short_name`, `display`, `icons`, `scope`, `start_url`, `file_handlers`, `share_target`, etc.) tailor how the OS installs and launches your app. ([Manifest reference](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest#members))
* **Service worker APIs** (install/activate/fetch events, Cache, Clients, FetchEvent, Background Sync, Periodic Sync, Background Fetch) run offline and keep your experience reliable. ([Service Worker API guide](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API))
* **OS integrations** such as the Notifications API, Badging API, Web Share API, Window Controls Overlay, and Web Share Target/File Handlers expand discoverability and feel-native. Each API is documented on MDN’s PWA reference list.

### 0.5. Auditing & observability
Use Lighthouse (Progressive Web App, Accessibility, Performance, SEO pillars) as the canonical audit; keep an eye on Core Web Vitals and measure real users through PageSpeed Insights or CrUX. Chrome’s `developer.chrome.com/docs/lighthouse` and `web.dev/explore/fast` paths explain how to interpret and improve those scores.

### 0.6. Official reference links
* [MDN Progressive web apps](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps)
* [MDN Best practices for PWAs](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Best_practices)
* [MDN Web app manifest reference](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest)
* [MDN Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
* [web.dev Progressive Web Apps](https://web.dev/progressive-web-apps/)
* [web.dev PWA checklist](https://web.dev/articles/pwa-checklist)
* [web.dev install criteria](https://web.dev/articles/install-criteria)
* [web.dev offline fallback page](https://web.dev/articles/offline-fallback-page)
* [web.dev permission UX](https://web.dev/articles/push-notifications-permissions-ux)
* [Chrome navigation management for PWAs](https://developer.chrome.com/docs/capabilities/pwa-navigation-management/)
* [W3C Web App Manifest spec](https://www.w3.org/TR/appmanifest/)
* [web.dev learn PWA course](https://web.dev/learn/pwa)
* [Chrome DevTools Lighthouse docs](https://developer.chrome.com/docs/lighthouse/)

---

## PART 1: CORE ARCHITECTURE & FIRST PRINCIPLES
Để thay thế Native App, PWA phải hoạt động theo mô hình **App Shell** và **Offline-First**.

### 1.1. First Principles vs. Firebase Abstraction
* **Asset Delivery:**
    * *Manual:* Phải tự cấu hình Server (Nginx/Apache), nén Gzip/Brotli, cấu hình SSL, Cache-Control headers thủ công.
    * *Firebase Hosting:* Tự động hóa toàn bộ. Global CDN, HTTP/2 & HTTP/3, Auto SSL, Atomic Deployments.
* **Database Sync:**
    * *Manual:* Phải viết API (REST/GraphQL), dựng WebSocket server (Socket.io), tự xử lý reconnect logic, tự viết logic lưu vào IndexedDB khi mất mạng và merge lại khi có mạng (conflict resolution).
    * *Firebase Firestore:* SDK xử lý toàn bộ qua protocol gRPC/WebChannel. `enableIndexedDbPersistence` tự động cache.
* **Authentication:**
    * *Manual:* Tự quản lý Session, JWT, HttpOnly Cookies, Refresh Token rotation, OAuth flows.
    * *Firebase Auth:* SDK tự động quản lý token lifecycle, lưu trữ an toàn trong IndexedDB/LocalStorage, tự động refresh token ngầm.

---

## PART 2: MANIFEST & CONFIGURATION (The Native Identity)

### 2.1. Web App Manifest (`manifest.json`)
Cấu hình tối đa để OS nhận diện như Native App.

```json
{
  "name": "SuperApp",
  "short_name": "SuperApp",
  "start_url": "/?source=pwa",
  "display": "standalone", 
  "background_color": "#121212",
  "theme_color": "#121212",
  "orientation": "portrait-primary",
  "icons": [
    { "src": "/icons/icon-192.png", "type": "image/png", "sizes": "192x192", "purpose": "any maskable" },
    { "src": "/icons/icon-512.png", "type": "image/png", "sizes": "512x512", "purpose": "any maskable" }
  ],
  "shortcuts": [ // Quick Actions (Force Touch / Right Click)
    {
      "name": "New Chat",
      "url": "/chat/new",
      "icons": [{ "src": "/icons/plus.png", "sizes": "96x96" }]
    }
  ],
  "share_target": { // Nhận dữ liệu từ app khác
    "action": "/share-target",
    "method": "POST",
    "enctype": "multipart/form-data",
    "params": {
      "title": "title", "text": "text", "url": "link",
      "files": [{ "name": "media", "accept": ["image/*", "application/pdf"] }]
    }
  },
  "protocol_handlers": [ // Deep linking (e.g., web+app://open/123)
    { "protocol": "web+superapp", "url": "/open?id=%s" }
  ],
  "related_applications": [ // Gợi ý cài native app nếu có (hoặc chặn gợi ý để ưu tiên PWA)
    { "platform": "webapp", "url": "https://myapp.com/manifest.json" }
  ]
}
```

Đoạn manifest mẫu trên tuân thủ [web.dev install criteria](https://web.dev/articles/install-criteria): phải phục vụ qua HTTPS, có `display`/`start_url` nằm trong `scope`, cung cấp icon 192/512 (maskable khi có thể) và được service worker đang hoạt động kiểm soát để trình duyệt coi là installable. Cú pháp chính xác cho `share_target` và `file_handlers` lấy theo [MDN share_target reference](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest/Reference/share_target) and [MDN file_handlers reference](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest/Reference/file_handlers).

> **Lưu ý về kích thước Icon cho Shortcuts:** Shortcuts mặc định có thể (và nên) tái sử dụng `icon-192.png` theo khuyến nghị của Chrome. Không cần thiết phải xuất một file size riêng (như 96x96) cho mục đích này trừ khi shortcut của bạn có artwork biểu tượng hoàn toàn riêng biệt (điều này nằm ngoài phạm vi của Favicon Generator).

### 2.2. iOS/Safari Meta Tags (Bắt buộc)
Safari không đọc hết manifest, phải khai báo thủ công trong `index.html`.
```html
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
<meta name="apple-mobile-web-app-title" content="SuperApp">
<link rel="apple-touch-icon" href="/icons/apple-icon-180.png">
```

---

### 2.3. ICON STRATEGY: BỘ TỐI THIỂU CHO PWA + SEO 🎯

#### 2.3.1. Bộ 5 File Cốt Lõi (Minimum Production Set)

Dựa trên [web.dev install criteria](https://web.dev/articles/install-criteria) và [Lighthouse PWA audit](https://developer.chrome.com/docs/lighthouse/pwa/installable-manifest/), đây là bộ tối thiểu production-ready:

```
/public/
├── icon-192.png          [192×192]   ← PWA Android minimum (BẮT BUỘC)
├── icon-512.png          [512×512]   ← PWA splash screen (BẮT BUỘC)
├── apple-icon-180.png    [180×180]   ← iOS home screen (BẮT BUỘC)
├── favicon.svg                       ← Modern browsers (KHUYẾN NGHỊ)
└── og-image.png          [1200×630]  ← Social sharing (KHUYẾN NGHỊ)
```

**Lý do 5 file này không thể thay thế nhau:**
- **192px & 512px**: Lighthouse yêu cầu tối thiểu 2 size này ([PWA installability](https://developer.chrome.com/docs/lighthouse/pwa/installable-manifest/#icons))
- **180px**: iOS Safari không đọc manifest → phải dùng meta tag riêng
- **SVG**: Scalable cho mọi zoom level, hỗ trợ dark mode
- **1200×630**: Chuẩn Open Graph cho Facebook/Twitter/LinkedIn

---

#### 2.3.2. Chi Tiết Từng File

##### **1. Icon 192×192 - PWA Core Icon**
```json
// manifest.json
{
  "icons": [
    { 
      "src": "/icon-192.png", 
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any maskable"  // ← 1 file phục vụ 2 mục đích
    }
  ]
}
```

**Tại sao 192px?**
- ✅ Lighthouse **bắt buộc** có ít nhất 1 icon ≥192px
- ✅ Android hiển thị rõ nét trên mọi màn hình (MDPI → XXXHDPI)
- ✅ Chrome sử dụng cho task switcher, notification badges

**`purpose: "any maskable"` giải thích:**
- `any`: Icon thường (hiển thị full design)
- `maskable`: Adaptive icon (Android crop thành hình tròn/vuông theo system theme)
- Dùng **CẢ HAI** = 1 file phục vụ 2 mục đích → giảm file count
- [MDN purpose reference](https://developer.mozilla.org/en-US/docs/Web/Manifest/icons#purpose)

---

##### **2. Icon 512×512 - Splash Screen & Store Listing**
```json
{
  "icons": [
    { 
      "src": "/icon-512.png", 
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any maskable"
    }
  ]
}
```

**Tại sao 512px?**
- ✅ Android splash screen (màn hình chờ khi mở app)
- ✅ Chrome Web Store thumbnail (nếu publish)
- ✅ Google Play Store yêu cầu 512px cho Trusted Web Activity
- ⚠️ **Không thể bỏ**: Lighthouse sẽ warning nếu thiếu

---

##### **3. Apple Touch Icon 180×180 - iOS Home Screen**
```html
<!-- index.html -->
<link rel="apple-touch-icon" href="/apple-icon-180.png">
```

**Tại sao 180px?**
- ✅ iPhone hiển thị icon trên Home Screen (retina @3x: 60×3=180)
- ✅ Safari không đọc manifest → **BẮT BUỘC** dùng meta tag riêng
- ❌ **Không thể dùng chung với Android**: iOS crop theo cách riêng
- [Apple Web App Icon guidelines](https://developer.apple.com/design/human-interface-guidelines/app-icons)

**Lưu ý thiết kế:**
- iOS tự bo tròn góc → logo phải để safe area 40px padding
- Không cần làm 120px, 152px, 167px nữa → iOS tự scale từ 180px

---

##### **4. SVG Favicon - Modern Browser Icon**
```html
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<link rel="icon" type="image/png" href="/icon-192.png"> <!-- Fallback -->
```

**Tại sao SVG?**
- ✅ 1 file cho mọi zoom level (16px → 256px)
- ✅ Hỗ trợ dark mode:
  ```svg
  <svg xmlns="http://www.w3.org/2000/svg">
    <style>
      @media (prefers-color-scheme: dark) {
        path { fill: white; }
      }
    </style>
    <path fill="black" d="..."/>
  </svg>
  ```
- ✅ File size nhỏ (~1-2KB vs PNG 10-20KB)
- ✅ Chrome 94+, Firefox 41+, Safari 14+ ([Can I Use SVG Favicons](https://caniuse.com/link-icon-svg))

**Legacy fallback:**
- Dùng lại `icon-192.png` → browser tự scale (acceptable quality)
- Hoặc thêm `favicon-32.png` nếu muốn pixel-perfect tabs

---

##### **5. Open Graph Image 1200×630 - Social Sharing**
```html
<!-- Open Graph (Facebook, LinkedIn, Discord) -->
<meta property="og:image" content="https://yourdomain.com/og-image.png">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">

<!-- Twitter Card -->
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:image" content="https://yourdomain.com/og-image.png">
```

**Tại sao 1200×630?**
- ✅ Chuẩn [Facebook/Open Graph](https://developers.facebook.com/docs/sharing/webmasters/images/) (ratio 1.91:1)
- ✅ LinkedIn, Discord, Slack đều dùng chung
- ✅ Twitter Large Card (tối thiểu 300×157, khuyến nghị 1200×630)
- ⚠️ **File size <1MB**, khuyến nghị <300KB
- [Open Graph Protocol](https://ogp.me/)

**Alternative 1200×1200:**
- Nếu ưu tiên Instagram/WhatsApp → dùng square 1:1
- Nhưng sẽ bị crop trên Facebook/Twitter → chọn 630 safer

---

#### 2.3.3. Các Size Có Thể Bỏ Qua

| Size | Platform | Lý do có thể skip |
|------|----------|-------------------|
| 16×16, 32×32 | Favicon legacy | SVG + 192 fallback đủ |
| 120×120, 152×167 | iOS old | iOS scale từ 180px |
| 144×144, 384×384 | Android legacy | Chrome scale từ 192/512 |
| 96×96 | Desktop shortcut | Ít được dùng, fallback 192 |

---

#### 2.3.4. Hướng Dẫn Thiết Kế Maskable Icon

**Vấn đề:** Android adaptive icons crop thành hình tròn/squircle → mất logo ở góc.

**Giải pháp Safe Zone:**
```
┌─────────────────────┐
│       512×512       │
│  ┌───────────────┐  │
│  │   Safe Zone   │  │ ← 80% center (410×410)
│  │   [Logo đây]  │  │ ← Đảm bảo không bị crop
│  └───────────────┘  │
└─────────────────────┘
     ↑ Bleed area (padding 51px mỗi cạnh)
```

**Design checklist:**
1. Logo/text phải nằm trong vùng 80% trung tâm
2. Background phải extend đến 4 góc (không để transparent)
3. Test trên nhiều shapes: Circle, Squircle, Rounded Square

**Test tool:** [Maskable.app Editor](https://maskable.app/editor) - Upload icon để xem preview trên mọi shape

---

#### 2.3.5. Automation Script

Tạo tất cả từ 1 master PNG/SVG 1024×1024:

```bash
# ImageMagick batch resize
convert master.png -resize 192x192 icon-192.png
convert master.png -resize 512x512 icon-512.png
convert master.png -resize 180x180 apple-icon-180.png
convert master.png -resize 1200x630 -background white -gravity center -extent 1200x630 og-image.png

# Optimize PNG size
optipng -o7 *.png
# hoặc: pngquant --quality=80-95 *.png
```

**Node.js automation:**
```javascript
// npm install sharp
const sharp = require('sharp');

const sizes = [
  { input: 'master.png', output: 'icon-192.png', width: 192, height: 192 },
  { input: 'master.png', output: 'icon-512.png', width: 512, height: 512 },
  { input: 'master.png', output: 'apple-icon-180.png', width: 180, height: 180 },
  { 
    input: 'master.png', 
    output: 'og-image.png', 
    width: 1200, 
    height: 630,
    fit: 'contain',
    background: { r: 255, g: 255, b: 255, alpha: 1 }
  }
];

sizes.forEach(({ input, output, width, height, fit, background }) => {
  sharp(input)
    .resize(width, height, { fit: fit || 'cover', background })
    .toFile(output)
    .then(() => console.log(`✅ ${output}`))
    .catch(err => console.error(`❌ ${output}:`, err));
});
```

**Online tools:**
- [PWA Asset Generator](https://github.com/elegantapp/pwa-asset-generator) (CLI tool)
- [Favicon Generator](https://realfavicongenerator.net/) (Web-based)
- [Maskable.app](https://maskable.app/) (Adaptive icon tester)

---

#### 2.3.6. Validation Checklist

**Lighthouse Audit (Chrome DevTools):**
```bash
✅ manifest.json có ít nhất 192px + 512px icon
✅ Icons có purpose="any" hoặc "maskable"
✅ Apple touch icon trong <head>
✅ Favicon hiển thị đúng
✅ Lighthouse PWA score ≥ 90/100
```

**Cross-Platform Test:**
```bash
✅ Android Chrome: Menu → "Install app" / "Add to Home screen"
✅ iOS Safari: Share button → "Add to Home Screen"
✅ Desktop Chrome: Address bar có install icon (⊕)
✅ Social: Paste link vào Facebook/Twitter → preview đúng og-image
```

**Manual inspection:**
```bash
# Check manifest
curl https://yourdomain.com/manifest.json | jq '.icons'

# Check og:image size
curl -I https://yourdomain.com/og-image.png | grep Content-Length

# Validate manifest
npx web-manifest-validator manifest.json
```

---

#### 2.3.7. Manifest.json Hoàn Chỉnh với Icon Strategy

```json
{
  "name": "SuperApp",
  "short_name": "SuperApp",
  "start_url": "/?source=pwa",
  "display": "standalone",
  "background_color": "#121212",
  "theme_color": "#121212",
  "orientation": "portrait-primary",
  "icons": [
    {
      "src": "/icon-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any maskable"
    },
    {
      "src": "/icon-512.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any maskable"
    }
  ]
}
```

**HTML Head Tags Hoàn Chỉnh:**
```html
<!DOCTYPE html>
<html lang="vi">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  
  <!-- PWA Manifest -->
  <link rel="manifest" href="/manifest.json">
  <meta name="theme-color" content="#121212">
  
  <!-- iOS Meta Tags -->
  <meta name="apple-mobile-web-app-capable" content="yes">
  <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
  <meta name="apple-mobile-web-app-title" content="SuperApp">
  <link rel="apple-touch-icon" href="/apple-icon-180.png">
  
  <!-- Favicons -->
  <link rel="icon" type="image/svg+xml" href="/favicon.svg">
  <link rel="icon" type="image/png" href="/icon-192.png">
  
  <!-- Open Graph / Social Sharing -->
  <meta property="og:title" content="SuperApp - Your Tagline">
  <meta property="og:description" content="App description for social sharing">
  <meta property="og:image" content="https://yourdomain.com/og-image.png">
  <meta property="og:image:width" content="1200">
  <meta property="og:image:height" content="630">
  <meta property="og:url" content="https://yourdomain.com">
  <meta property="og:type" content="website">
  
  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:title" content="SuperApp - Your Tagline">
  <meta name="twitter:description" content="App description for Twitter">
  <meta name="twitter:image" content="https://yourdomain.com/og-image.png">
  
  <title>SuperApp</title>
</head>
<body>
  <!-- App content -->
</body>
</html>
```

---

#### 2.3.8. Tóm Tắt: Priority Map

```
CRITICAL (Không thể thiếu - Lighthouse yêu cầu):
├── 192×192 PNG (PWA Android, purpose="any maskable")
├── 512×512 PNG (Splash screen, purpose="any maskable")
└── 180×180 PNG (iOS Safari, apple-touch-icon)

HIGHLY RECOMMENDED (SEO/Discovery):
├── favicon.svg (Modern browsers, dark mode support)
└── 1200×630 PNG (Social sharing, og:image)

OPTIONAL (Nice to have):
└── 32×32 PNG (Legacy favicon fallback cho IE11)
```

**Kết luận:** Với **5 files** (192, 512, 180, SVG, 1200×630), bạn có bộ production-ready đáp ứng:
- ✅ Lighthouse 100/100 PWA score
- ✅ iOS + Android installability
- ✅ SEO social previews (Facebook, Twitter, LinkedIn)
- ✅ Modern browser support (Chrome, Firefox, Safari, Edge)

Đây là bộ tối thiểu được [web.dev](https://web.dev/articles/install-criteria), [MDN](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest#icons), and [Lighthouse](https://developer.chrome.com/docs/lighthouse/pwa/installable-manifest/) khuyến nghị cho production 2025. 🚀

---

## PART 3: FILE HANDLER INTEGRATION 🔥 PRODUCTION LESSON

#### 2.4.1. Hai Điểm Vào: Share Target vs File Handlers

**QUAN TRỌNG:** Để PWA trở thành trình xử lý file đầy đủ trên Android, bạn cần **CẢ HAI**:

| Feature | Manifest Key | Xuất Hiện Khi | Platform Support |
|---------|--------------|---------------|------------------|
| **Share Target** | `share_target` | User share file từ app khác → "Share to..." | Android, Desktop |
| **File Handlers** | `file_handlers` | User long-press file → "Open With..." | Android (Chrome 102+), Desktop |

**❌ Sai lầm thường gặp:** Chỉ có `share_target` → App chỉ nhận được share, không xuất hiện trong "Open With".

---

#### 2.4.2. Share Target - Cú Pháp Hiện Đại (Android 12+)

**❌ CÚ PHÁP CŨ (DEPRECATED - KHÔNG HOẠT ĐỘNG):**
```json
{
  "share_target": {
    "action": "/share-target",
    "files": [  // ❌ SAI: files không được nằm ở đây
      { "name": "media", "accept": ["audio/*"] }
    ]
  }
}
```

**✅ CÚ PHÁP ĐÚNG (PRODUCTION-TESTED):**
```json
{
  "share_target": {
    "action": "/share-target",
    "method": "POST",
    "enctype": "multipart/form-data",
    "params": {  // ✅ BẮT BUỘC: Phải wrap trong params object
      "title": "title",
      "text": "text",
      "url": "link",
      "files": [
        {
          "name": "media",
          "accept": [
            // ✅ CHI TIẾT QUAN TRỌNG: Liệt kê cụ thể thay vì chỉ generic
            "audio/*",       // Generic fallback
            "audio/mpeg",    // Cụ thể giúp Android Intent matching tốt hơn
            "audio/wav",
            "audio/mp4",
            "audio/aac",
            "audio/flac",
            "audio/ogg",
            "audio/webm",
            "audio/x-m4a",
            "image/*",
            "image/jpeg",
            "image/png",
            "image/webp",
            "image/gif",
            "video/*",
            "video/mp4",
            "video/webm",
            "video/quicktime",
            "application/pdf"
          ]
        }
      ]
    }
  }
}
```

**💡 TẠI SAO PHẢI CHI TIẾT `accept`?**
1. Android sử dụng MIME types để match Intent Filter.
2. Generic type (`audio/*`) chỉ là fallback, **không đủ mạnh** để OS ưu tiên app của bạn.
3. Càng chi tiết → Android càng **"tự tin"** rằng app xử lý được file đó.

---

#### 2.4.3. File Handlers (Open With... Integration)

Tính năng này giúp PWA xuất hiện khi user **long-press file trong File Manager** → "Open With...".

```json
{
  "file_handlers": [
    {
      "action": "/open-file",
      "accept": {
        // ⚠️ Cú pháp KHÁC với share_target: Object thay vì Array
        "audio/mpeg": [".mp3"],
        "audio/wav": [".wav"],
        "audio/aac": [".aac", ".m4a"],
        "image/png": [".png"],
        "image/jpeg": [".jpg", ".jpeg"]
      }
    }
  ]
}
```

MDN keeps the authoritative syntax for `file_handlers` and the compatible platforms (Android Chrome 102+ and desktop choosers) in [its File Handlers reference](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Manifest/Reference/file_handlers), so mirror the `accept` map above and add any additional MIME/extension pairs you need. The same section also explains how `share_target` ties into Android’s share sheet experience.
