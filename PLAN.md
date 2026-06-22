# PLAN.md — Mở rộng bộ Favicon: SEO/Discovery sizes + Fill Customization

> Đúc kết quyết định kiến trúc từ phiên thảo luận cố vấn. Chưa code, đây là plan để review trước khi triển khai.

---

## 1. Vấn đề hiện tại

- Không có size nào trong khoảng 32–180px phục vụ tốt: footer cross-link (24 CSS × DPR), Google SERP favicon (yêu cầu chính thức >48px), browser tab hiện đại, RSS reader.
- `favicon.ico` (16/32) hiện resize thẳng từ ảnh gốc — không ép nền (đúng tinh thần), nhưng tài liệu cũ (`master.md`) mô tả sai là có bước "extract symbol/remove background". **Bước này không tồn tại trong code và sẽ KHÔNG được implement** — background-removal tự động không đủ tin cậy để liều.
- `theme_color` / `background_color`: hạ tầng Rust đã hỗ trợ override (`FaviconOptions.with_theme_color/with_background_color`) nhưng **không có UI nào expose** — cả demo lẫn akitao.com đều chỉ hiển thị tĩnh, không cho sửa.
- Fill cho nhóm icon bắt buộc đục nền (512-maskable, apple-touch-icon) đang **chỉ tô màu đặc** (`fit_safe_zone()` trong `transform.rs`), không có khái niệm gradient. Khi không detect được màu nền, fallback cứng về `#000000`.

---

## 2. Nguyên tắc cốt lõi: 2 nhóm artifact, 2 cơ chế xử lý

Không tự động extract/remove background. Thay vào đó: **tool tôn trọng transparency của input, không tự ý tạo ra nó.** Khuyến khích người dùng upload PNG nền trong suốt (symbol-only) để đạt kết quả tốt nhất ở Nhóm A.

| Nhóm | File | Cơ chế | Lý do |
|---|---|---|---|
| **A — Source-faithful** (không ép nền) | `favicon.ico` (16/32)<br>`icon-192.png`<br>`icon-48.png` *(mới)*<br>`icon-96.png` *(mới)* | `resize_exact()` — chỉ Lanczos3 resize, giữ nguyên alpha kênh gốc 100% | Nếu input transparent → output transparent. Nếu input có nền đục → output giữ nguyên nền đó. Tool không nói dối, không tự suy diễn. |
| **B — Force-filled** (bắt buộc đục nền) | `icon-512-maskable.png`<br>`apple-touch-icon.png` | `fit_safe_zone()` — tạo canvas màu, overlay logo vào safe-zone 80% | Bắt buộc theo spec OS: Android maskable không chấp nhận transparency trong safe area hiệu quả; iOS tự tô đen các góc trong suốt (bug đã biết) |

**Hệ quả:** `icon-48.png` và `icon-96.png` dùng chung hạ tầng với `icon-192.png` (đã đúng sẵn, không cần sửa logic) — chỉ thêm lệnh gọi `resize_exact(&img, 48)` / `resize_exact(&img, 96)` trong `lib.rs`.

---

## 3. Size chốt cho nhóm SEO/Discovery: 48 + 96

| Size | Lý do |
|---|---|
| **48×48** | Ngưỡng tối thiểu chính thức Google Search Central khuyến nghị cho favicon hiện trong SERP (>48px, nên là bội số 48). Cũng là size ví dụ chính thức cho browser tab hiện đại (Chrome/Edge retina), RSS feed reader. |
| **96×96** | Bội số 48 tiếp theo — phủ DPR 3x-4x cho các ngữ cảnh hiển thị nhỏ (vd. footer cross-link 24px CSS). |

Không chọn 32 (dưới ngưỡng khuyến nghị Google) hay 64 (không nằm trong convention chính thức nào). Cặp 48/96 đi theo đúng pattern any/maskable (192/512) đã có sẵn trong hệ thống.

**Định danh:** 48/96 thuộc category "SEO & Brand Discovery" — KHÔNG đưa vào `manifest.json` (không phục vụ installability), chỉ khai báo qua `<link rel="icon">` trong HTML `<head>`. Tránh gộp vào câu chuyện marketing "5 file Universal PWA" hiện có — đây là nhóm tính năng cộng thêm, độc lập, không làm hỏng USP cũ.

> **Lưu ý quan trọng — KHÔNG dùng lý do PWA shortcut để justify size 96:** `manifest.shortcuts` chỉ kích hoạt khi đã cài PWA (long-press icon trên launcher) — ngữ cảnh hoàn toàn khác footer/SERP. Theo khuyến nghị chính thức của Google (web.dev), nếu không cần pixel-perfect cho shortcut thì dùng lại `icon-192.png` có sẵn là đủ — generator đã giải xong nhu cầu tối thiểu này miễn phí, không cần thêm size riêng. 96 tồn tại vì lý do footer/SERP độc lập; việc nó tiện dùng tạm cho shortcut chỉ là bonus phụ.

---

## 4. Color customization — chỉ cần nối UI vào hạ tầng có sẵn

`FaviconOptions` trong `lib.rs` đã có `with_theme_color()` / `with_background_color()` hoạt động đầy đủ. Vấn đề 100% nằm ở tầng UI (Vue) — chưa có input nào, chỉ hiển thị tĩnh.

**Đề xuất UI:** `<input type="color">` native đặt cạnh swatch hiện có, đồng bộ 2 chiều với 1 ô text hex bên cạnh (picker trực quan + gõ tay khi cần chính xác). Khi đổi màu → gọi lại `generate_favicon_set` với options đã set. Không cần thư viện color-picker ngoài.

---

## 5. Fill color mặc định đen + Gradient toggle (Hiệu ứng nền lồi 3D)

- **Fill color:** tái dùng đúng field `background_color` đã có (không cần field Rust mới) — chỉ cần UI cho sửa (mục 4). Mặc định vẫn `#000000` khi không detect được màu (giữ nguyên fallback hiện tại trong `color.rs`).
- **Gradient toggle (Mới - Hiệu ứng 3D):** `fit_safe_zone()` hiện chỉ tô màu đặc. Thêm field `fill_gradient: bool` và `gradient_color: String` vào `FaviconOptions`.
  - **Thuật toán:** Nguồn sáng chiếu vào từ góc top-left (khoảng 35 độ) tạo thành hiệu ứng bề mặt lồi (convex). Tâm phản quang lệch về góc trên-trái (`hx = cx - size * 0.25`, `hy = cy - size * 0.25`).
  - **Mức độ lan toả:** Ánh sáng phản quang cực rộng và cực mạnh (`max_dist = size * 1.5`, áp dụng hàm giảm suy hao `powf(1.8)`) giúp tăng chiều sâu tối đa.
  - **Highlight Color Picker:** UI cho phép người dùng tự do tuỳ chỉnh màu phản quang (`gradient_color`), kết hợp với màu nền (`background_color`) sinh ra gradient chân thực, sâu sắc hơn.
  - Khi `fill_gradient = false` → fallback tô đặc 1 màu như hiện tại.
- **Phạm vi áp dụng:** CHỈ Nhóm B (512-maskable, apple-touch-icon). Không bao giờ áp vào Nhóm A — vi phạm nguyên tắc "tôn trọng nguyên trạng input" ở mục 2.

---

## 6. Việc cần làm — Rust (`src/`)

| File | Thay đổi |
|---|---|
| `src/lib.rs` | Thêm field `fill_gradient: bool` (default `true`) và `gradient_color: Option<String>` vào `FaviconOptions`. Thêm 2 field `icon_48`, `icon_96` vào `FaviconSet`, build bằng `resize_exact(&img, 48)` / `resize_exact(&img, 96)` (Nhóm A, không qua `fit_safe_zone`). |
| `src/transform.rs` | Thêm tham số `gradient: bool` và `grad_rgb` vào `fit_safe_zone()`, triển khai thuật toán radial gradient tâm lệch top-left 35 độ tạo hiệu ứng 3D convex light reflection. |
| `src/color.rs` | Không cần đổi logic detect — chỉ đảm bảo `hex_to_rgb()` được tái dùng đúng cho cả `theme_color` lẫn `background_color` override từ UI. |

---

## 7. Việc cần làm — `demo/src/App.vue`

- Thêm color picker (theme + background), đồng bộ hex input, gọi lại WASM khi đổi.
- Thêm toggle "Gradient fill" (default ON) cạnh khu cấu hình.
- Thêm 2 preview mới cho `icon-48`/`icon-96` (vd. mockup "Browser Tab Retina" hoặc "SERP Discovery").
- Cập nhật bảng `outputArtifacts`: thêm 2 dòng mới + cột/badge phân biệt "Source-faithful" vs "Auto-filled" theo đúng Nhóm A/B ở mục 2.
- Cập nhật `buildSnippet()` thêm `<link rel="icon" sizes="48x48">` và `sizes="96x96"`. KHÔNG thêm vào `buildManifest()`.
- Cập nhật `createZip()` thêm `favicon/icon-48.png`, `favicon/icon-96.png`.
- Thêm hint text gần dropzone: khuyến khích upload PNG nền trong suốt để có kết quả tốt nhất ở favicon.ico/48/96.

---

## 8. Việc cần làm — `akitao.com/.../favicon-generator.vue`

- Y hệt mục 7, theo đúng cấu trúc i18n (`copy.vi` / fallback en) hiện có.
- Cập nhật `outputArtifactsDetails` (cả vi/en): thêm 2 hàng `icon-48.png`, `icon-96.png`; sửa lại cột "purpose" của `favicon.ico` — bỏ cụm "nền đục" gây hiểu nhầm là tool tự ý ép nền (theo nguyên tắc mục 2, ICO là Nhóm A — source-faithful).
- Cập nhật `compRows` / `exclusionsItems` nếu cần để tránh tự mâu thuẫn với USP "chỉ 5 file" — đề xuất giữ nguyên con số 5 cho nhóm PWA-install core, giới thiệu 48/96 là tính năng "SEO & Brand Discovery" riêng biệt, không gộp đếm vào 5.
- Thêm UI cho color picker (Theme, Background, Highlight) + toggle Hiệu ứng 3D giống mục 7.

---

## 9. Việc cần làm — Docs

| File | Thay đổi |
|---|---|
| `docs/philosophy/master.md` | Xoá mọi cụm mô tả "extract symbol/remove background" tự động (không tồn tại, sẽ không làm). Viết lại rõ bảng Nhóm A/B ở mục 2. Thêm khuyến nghị input nên transparent. |
| `docs/feat/artifact-matrix.md` | Thêm 2 hàng `icon-48.png`, `icon-96.png` vào category riêng "SEO & Brand Discovery" — ghi rõ không thuộc bộ PWA-installability 4-file, không khai trong `manifest.icons`. |
| `docs/ref/PWA.md` | Thêm ghi chú: shortcuts mặc định reuse `icon-192.png` theo khuyến nghị Chrome; không cần size riêng trừ khi có per-action artwork riêng (ngoài phạm vi generator). |

---

## 10. Không làm (giữ nguyên scope)

- Không tự động xoá/tách nền (background removal AI hay thuật toán heuristic) — quá rủi ro.
- Không thêm size riêng cho PWA shortcut — dùng lại `icon-192.png` theo khuyến nghị chính thức.
- Không phá vỡ USP "5 file Universal" của nhóm PWA-install core (192/512/180/ico/manifest) — 48/96 là nhóm tính năng cộng thêm, tách bạch rõ trong UI và docs.

---

## 11. Gaps đã rà soát & bổ sung (review lần 2)

| # | Gap | Mức độ | Hướng xử lý |
|---|---|---|---|
| 1 | **Build & sync pipeline thiếu hoàn toàn.** `Makefile` chỉ build ra `pkg/`, không sync sang `akitao.com/app/wasm/aki-favicon-generator/`. | Chặn deploy | Thêm bước copy thủ công/script sau mỗi `make build`, hoặc thêm target `make sync-akitao` trong Makefile. |
| 2 | **"HSL lighten/darken" chưa có hạ tầng.** `color.rs` chỉ có `rgb_to_hsv()`, không có chiều ngược `hsv_to_rgb`/`hsl_to_rgb`. | Kỹ thuật | Chọn 1 trong 2: (a) viết đủ roundtrip HSL, hoặc (b) lerp thẳng trong RGB-space về trắng/đen — đơn giản hơn, đúng tinh thần Zero-Bloat. **Khuyến nghị (b).** |
| 3 | **wasm-bindgen builder pattern consume `self`.** Gọi `with_*()` 1 lần sẽ tiêu thụ object JS gốc — gọi tiếp trên object cũ sẽ panic. | Kỹ thuật, dễ vỡ khi code thật | Luôn build `FaviconOptions` mới, chain đầy đủ 1 statement mỗi lần `generate_favicon_set`, không giữ instance qua nhiều event handler. |
| 4 | **Thiếu chỗ lưu `imageBytes` gốc** để regenerate khi đổi màu/gradient sau upload — hiện chỉ giữ object URL preview. | Kiến trúc UI | Thêm `ref` lưu raw bytes từ lần upload đầu, dùng lại mỗi khi settings đổi. |
| 5 | **Thiếu debounce cho color input** — kéo picker bắn event liên tục. | UX/performance | Bind `@change` (commit) thay vì `@input`, hoặc debounce 150–300ms. |
| 6 | **ICO rounded-corner (16% radius) vẫn baked-in** trong `apply_rounded_corners()` — mâu thuẫn "Shape Fallacy" principle đã nêu ở `master.md`. Từng nêu ở thảo luận trước nhưng bị sót khỏi plan. | Backlog, không chặn đợt này | Ghi nhận nợ kỹ thuật, xử lý ở đợt sau — KHÔNG để icon-48/96 dùng chung pipeline ICO này. |
| 7 | **HTML snippet thiếu cú pháp chính xác.** | Nhỏ | Dòng mới: `<link rel="icon" type="image/png" sizes="48x48" href="/favicon/icon-48.png">` và tương tự cho 96x96 — theo đúng format dòng 192 hiện có. |

---

## 12. Acceptance checklist trước khi merge

- [ ] Input PNG nền trong suốt → `favicon.ico`/`icon-48`/`icon-96`/`icon-192` đều giữ alpha, không bị tô đen/tô trắng ngoài ý muốn.
- [ ] Input ảnh nền đục (JPEG hoặc PNG full-bleed) → cùng nhóm A giữ nguyên nền gốc, không crash, không tự ý thêm transparency giả.
- [ ] `icon-512-maskable.png` / `apple-touch-icon.png` với `fill_gradient = true` (default) → có gradient chéo nhẹ, không bị banding rõ ở mắt thường.
- [ ] `fill_gradient = false` → fallback đúng về tô đặc 1 màu như hành vi cũ.
- [ ] Đổi `theme_color`/`background_color` qua UI → regenerate đúng mà không cần upload lại file.
- [ ] `manifest.json` xuất ra KHÔNG chứa icon-48/96.
- [ ] HTML snippet chứa đủ 2 dòng `<link>` mới, đúng cú pháp `type="image/png"`.
- [ ] `pkg/` build mới đã được sync sang akitao.com trước khi test trên `favicon-generator.vue` thật.
- [ ] Lighthouse PWA audit trên akitao.com vẫn pass sau khi thêm 2 file mới (không bị tính nhầm là bloat).
