# Transfigure — Engineering & Product Plan

> **Tagline:** Private-first file conversion. Everything runs in your browser. Your files never leave your machine.

---

## Table of Contents

1. [Vision & Positioning](#1-vision--positioning)
2. [Technical Architecture Overview](#2-technical-architecture-overview)
3. [Repository Structure](#3-repository-structure)
4. [Developer Environment (DevContainer)](#4-developer-environment-devcontainer)
5. [CI/CD Pipeline — GitHub → Cloudflare](#5-cicd-pipeline--github--cloudflare)
6. [Rust WASM Engine — Conversion Crate](#6-rust-wasm-engine--conversion-crate)
7. [Supported Conversions (v1 Roadmap)](#7-supported-conversions-v1-roadmap)
8. [Frontend Architecture](#8-frontend-architecture)
9. [Cloudflare Workers — Edge Backend](#9-cloudflare-workers--edge-backend)
10. [Monetisation & Licensing Layer](#10-monetisation--licensing-layer)
11. [Security Model](#11-security-model)
12. [Performance Strategy](#12-performance-strategy)
13. [Testing Strategy](#13-testing-strategy)
14. [Observability & Analytics](#14-observability--analytics)
15. [Domain, Branding & Launch](#15-domain-branding--launch)
16. [Phased Delivery Plan](#16-phased-delivery-plan)
17. [Cost Model Summary](#17-cost-model-summary)
18. [Open Questions & Decisions](#18-open-questions--decisions)

---

## 1. Vision & Positioning

### The Problem

Every major file conversion service (Smallpdf, ILovePDF, CloudConvert, Zamzar) uploads your files to a remote server. For anyone handling legal documents, medical records, financial data, or personal files, this is a fundamental trust and compliance problem. GDPR, HIPAA, and general privacy instincts push users to avoid these tools for sensitive work. They use them anyway because there's no real alternative.

### The Solution

A web application where the conversion engine runs entirely inside the user's browser as a compiled WebAssembly binary. Drag a file on, get a converted file back. Zero bytes leave the device. The server never sees the file at any point — not even encrypted. This is verifiable by the user via browser DevTools (no upload network requests).

### Competitive Moat

- **Privacy is structural, not a policy.** Competitors could say "we delete files immediately" but the upload still happens. We can say "the upload is architecturally impossible."
- **Works offline.** Once the WASM binary is cached by the service worker, the app functions with no internet connection at all.
- **Speed.** WASM runs at near-native speed, much faster than a round-trip to a remote server for small/medium files.
- **No account required** for free tier. Frictionless first use.

### Target Users (Priority Order)

1. Legal, finance, and healthcare professionals handling sensitive documents
2. Privacy-conscious individuals (developers, journalists, activists)
3. Corporate IT environments where data egress policies prohibit uploading files
4. Bulk converters who hit rate limits on free tiers of competitors

---

## 2. Technical Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    User's Browser                        │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │            Vanilla JS / Web Components           │    │
│  │         (TypeScript, no heavy framework)         │    │
│  └─────────────────┬───────────────────────────────┘    │
│                    │  File bytes (in memory)              │
│  ┌─────────────────▼───────────────────────────────┐    │
│  │          Rust WASM Conversion Engine             │    │
│  │   (wasm-pack, wasm-bindgen, web-sys)             │    │
│  │   Loaded lazily per conversion type              │    │
│  └─────────────────────────────────────────────────┘    │
│                    │                                      │
│  ┌─────────────────▼───────────────────────────────┐    │
│  │          Service Worker (Rust → WASM)            │    │
│  │    Asset caching, offline support, SW lifecycle   │    │
│  └─────────────────────────────────────────────────┘    │
└───────────────────────┬─────────────────────────────────┘
                        │ Only: (a) quota check, (b) JWT validation,
                        │ (c) Stripe webhook. NO file data.
┌───────────────────────▼─────────────────────────────────┐
│              Cloudflare Pages + Workers                   │
│                                                          │
│  Pages:    Serves static assets (.html, .wasm, .js)      │
│  Workers:  /api/quota   — check + increment daily limit  │
│            /api/license — issue + validate JWT token     │
│            /api/webhook — Stripe/Paddle payment events   │
│                                                          │
│  KV Store: { "quota:{fingerprint}": count }              │
│  KV Store: { "license:{token}": expiry + tier }          │
└─────────────────────────────────────────────────────────┘
```

**Key principle:** The Workers are a quota and licence gate only. They never receive, buffer, or proxy any file data. The conversion logic lives entirely in the WASM module.

---

## 3. Repository Structure

```
transfigure/
│
├── .devcontainer/
│   ├── devcontainer.json          # VSCode devcontainer config
│   └── Dockerfile                 # Ubuntu + Rust + wasm-pack + Node + wrangler
│
├── .github/
│   └── workflows/
│       ├── ci.yml                 # PR checks: fmt, clippy, test, wasm-pack build
│       └── deploy.yml             # main branch → Cloudflare Pages deploy
│
├── crates/
│   ├── converter/                 # Core conversion logic, compiles to WASM
│   │   ├── src/
│   │   │   ├── lib.rs             # wasm-bindgen entry points
│   │   │   ├── image.rs           # Image format conversions
│   │   │   ├── document.rs        # Markdown, HTML, plaintext conversions
│   │   │   ├── spreadsheet.rs     # CSV ↔ JSON, TSV transformations
│   │   │   ├── archive.rs         # ZIP creation/extraction
│   │   │   └── utils.rs           # Shared byte manipulation helpers
│   │   └── Cargo.toml
│   │
│   ├── service-worker/            # Service worker compiled to WASM via workers-rs
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   │
│   └── worker/                    # Cloudflare Worker (quota + licence) via workers-rs
│       ├── src/
│       │   ├── lib.rs
│       │   ├── quota.rs
│       │   ├── license.rs
│       │   └── webhook.rs
│       └── Cargo.toml
│
├── site/                          # Static frontend
│   ├── index.html
│   ├── app.ts                     # Main application logic (TypeScript)
│   ├── components/
│   │   ├── drop-zone.ts           # Drag-and-drop component
│   │   ├── format-picker.ts       # Conversion format selector
│   │   ├── progress-bar.ts        # WASM progress callbacks
│   │   └── paywall-modal.ts       # Upgrade prompt
│   ├── styles/
│   │   └── main.css               # CSS custom properties, no framework
│   └── pkg/                       # wasm-pack output (gitignored, build artefact)
│       └── ...
│
├── scripts/
│   ├── build.sh                   # Full build: wasm-pack + tsc + copy to site/pkg
│   ├── dev.sh                     # Wrangler dev server with hot reload
│   └── test-conversions.sh        # Golden file regression tests
│
├── tests/
│   ├── fixtures/                  # Sample input files for each format
│   └── golden/                    # Expected output files for regression
│
├── wrangler.toml                  # Cloudflare Worker + Pages config
├── Cargo.toml                     # Workspace root
├── package.json                   # wasm-pack, TypeScript, esbuild
└── README.md
```

---

## 4. Developer Environment (DevContainer)

The devcontainer ensures every contributor has an identical, reproducible environment with zero local setup beyond Docker and VSCode.

### `Dockerfile`

Base image: `mcr.microsoft.com/devcontainers/rust:1-bullseye`

Additional tooling installed at build time:
- `wasm-pack` (latest stable via installer script)
- `wasm-bindgen-cli` (pinned to match `Cargo.toml`)
- `wasm-opt` via `binaryen` (WASM size optimisation)
- `Node.js 20 LTS` + `npm` (TypeScript compilation, esbuild)
- `wrangler` (Cloudflare Workers CLI, via npm)
- `cargo-watch` (file watching for live rebuild during dev)
- `cargo-nextest` (faster test runner)
- `cargo-tarpaulin` (coverage)
- `clippy` and `rustfmt` (enforced in CI)
- `wabt` tools (`wasm2wat` etc.) for WASM inspection during debugging

Browser targets added to Rust toolchain:
```
rustup target add wasm32-unknown-unknown
```

### `devcontainer.json`

- VSCode extensions pre-installed: `rust-analyzer`, `Even Better TOML`, `Error Lens`, `Prettier`, `ESLint`, `Thunder Client` (API testing), `GitLens`
- Port forwarding: `8787` (wrangler dev), `3000` (optional static file server)
- Post-create command runs `scripts/build.sh` to produce an initial working build
- Environment variables: `.env` file mounted for `CLOUDFLARE_API_TOKEN`, `STRIPE_SECRET_KEY`, etc. (never committed)
- Remote user: non-root, matches UID/GID to avoid permission issues on Linux hosts

### Developer Workflow (Daily)

```bash
# Start devcontainer, then:
cargo watch -x "build -p converter --target wasm32-unknown-unknown"  # WASM hot rebuild
wrangler pages dev ./site --local                                      # Serves site locally
# Changes to site/ and crates/ trigger rebuild and browser refresh
```

---

## 5. CI/CD Pipeline — GitHub → Cloudflare

### Branch Strategy

- `main` — production, auto-deploys to Cloudflare Pages on every push
- `develop` — integration branch, deploys to a Cloudflare Pages preview environment
- Feature branches — deploy to ephemeral preview URLs via Cloudflare's automatic preview deployments

### `ci.yml` — Pull Request Checks

Triggered on every PR opened or updated against `main` or `develop`.

Steps:
1. `cargo fmt --check` — fails if formatting differs
2. `cargo clippy --all-targets -- -D warnings` — fails on any lint warning
3. `cargo test --workspace` — unit tests for non-WASM crates
4. `wasm-pack build crates/converter --target web` — confirms WASM compiles cleanly
5. `wasm-opt` size check — fails if `.wasm` binary exceeds configurable threshold (prevents accidental dependency bloat)
6. TypeScript type check via `tsc --noEmit`
7. Golden file regression tests via `scripts/test-conversions.sh` — runs the WASM build against fixture files in a headless Node environment using `@wasmer/wasi` or similar runner

### `deploy.yml` — Production Deployment

Triggered on push to `main`.

Steps:
1. Run full CI suite (same as above) — deploy blocked on any failure
2. `wasm-pack build crates/converter --target web --release` with `wasm-opt -O3`
3. `esbuild` bundles TypeScript → single `app.js`
4. Copy `pkg/` output into `site/pkg/`
5. `wrangler pages deploy ./site --project-name transfigure` using `CLOUDFLARE_API_TOKEN` secret
6. `wrangler deploy crates/worker` to deploy the Worker separately

The Cloudflare API token is stored as a GitHub Actions repository secret. No secrets are baked into any build artefact.

### Preview Deployments

Cloudflare Pages natively creates preview URLs for every non-main branch (e.g. `https://feature-xyz.transfigure.pages.dev`). These are linked back to the GitHub PR automatically via Cloudflare's GitHub integration, so reviewers can test the actual running app before merging.

---

## 6. Rust WASM Engine — Conversion Crate

### Key Design Principles

**Single flat API surface.** The WASM module exposes a small number of functions via `wasm-bindgen`. The JS side passes in raw bytes and a conversion config, and receives raw bytes back. No file paths, no filesystem, no sockets.

```rust
#[wasm_bindgen]
pub fn convert(input: &[u8], config: JsValue) -> Result<Box<[u8]>, JsValue>
```

`config` is a serialised JSON object specifying input format, output format, and any options (e.g. image quality, page range). Serde handles deserialisation inside the WASM.

**Progress callbacks.** For large files, the WASM calls back into JS via a `js_sys::Function` to report progress as a 0.0–1.0 float, which drives the progress bar UI.

**Panic handling.** `console_error_panic_hook` is enabled in debug builds so panics surface in the browser console. In release builds, panics produce a clean error result that JS can display gracefully, rather than crashing silently.

**No `std::fs`.** The crate is `#![forbid(unsafe_code)]` where possible (some crates may require unsafe; this is audited per dependency). WASM has no filesystem access, so file I/O goes through the byte slice API only.

### Dependency Philosophy

Every dependency is evaluated on three axes: (a) does it compile cleanly to `wasm32-unknown-unknown`, (b) what does it add to binary size, (c) how actively maintained is it. Dependencies that require `std::fs`, POSIX syscalls, or threading are avoided. The full dependency tree is reviewed before adding any new crate.

### Core Dependencies

| Crate | Purpose | WASM Support |
|---|---|---|
| `wasm-bindgen` | JS ↔ Rust bridge | Native |
| `web-sys` | Browser File API, console | Native |
| `js-sys` | JS types, callbacks | Native |
| `serde` + `serde_json` | Config deserialisation | Yes |
| `image` | Raster image encode/decode | Yes (no SIMD features) |
| `pulldown-cmark` | Markdown → HTML | Yes |
| `comrak` | Markdown → HTML (GitHub-flavoured, richer) | Yes |
| `lopdf` | PDF reading and manipulation | Partial — evaluate carefully |
| `docx-rs` | DOCX reading and writing | Yes |
| `calamine` | XLS/XLSX reading | Yes |
| `simple_excel_writer` | XLSX writing | Yes |
| `zip` | ZIP archive creation/extraction | Yes |
| `csv` | CSV parsing and writing | Yes |
| `base64` | Base64 encode/decode | Yes |
| `console_error_panic_hook` | Debug panic messages | Native |

### WASM Binary Size Strategy

A single monolithic WASM binary containing all conversion logic would be 10–30 MB, which is unacceptable for initial page load. Instead, the crate is split into **feature-gated modules** and built as separate WASM bundles:

- `converter-images.wasm` — all image format conversions (~3–5 MB)
- `converter-documents.wasm` — Markdown, HTML, plaintext (~1–2 MB)
- `converter-office.wasm` — DOCX, XLSX, PDF (~8–12 MB, lazy loaded)
- `converter-archive.wasm` — ZIP (~0.5 MB)

Each bundle is only fetched and instantiated when the user selects a conversion type that requires it. A conversion type can declare its module dependency, and the JS layer handles lazy instantiation with a loading indicator. Previously loaded modules are cached in the service worker so subsequent uses are instant.

`wasm-opt -O3 -Oz` (binaryen optimiser) is run on all release builds. Typically reduces binary size by 15–30%.

---

## 7. Supported Conversions (v1 Roadmap)

### Phase 1 (Launch)
High-confidence WASM implementations, excellent library support.

| From | To | Crate |
|---|---|---|
| PNG, JPG, JPEG, WebP, GIF, BMP, TIFF | Any of the above | `image` |
| Markdown (.md) | HTML | `comrak` |
| Markdown (.md) | Plain text (stripped) | `pulldown-cmark` |
| HTML | Markdown | custom parser |
| CSV | JSON | `csv` + `serde_json` |
| CSV | TSV | `csv` |
| JSON | CSV | `csv` + `serde_json` |
| Multiple files | ZIP | `zip` |
| ZIP | Extract files | `zip` |
| SVG | PNG | `resvg` (pure Rust SVG renderer) |
| Base64 | Binary | `base64` |
| Binary | Base64 | `base64` |

### Phase 2 (Post-launch, ~3 months)

| From | To | Crate / Approach |
|---|---|---|
| DOCX | Markdown | `docx-rs` + custom renderer |
| DOCX | Plain text | `docx-rs` |
| XLSX / XLS | CSV | `calamine` |
| XLSX / XLS | JSON | `calamine` + `serde_json` |
| CSV | XLSX | `simple_excel_writer` |
| Markdown | DOCX | `docx-rs` |
| JSON | YAML | `serde_yaml` |
| YAML | JSON | `serde_yaml` + `serde_json` |
| TOML | JSON | `toml` + `serde_json` |

### Phase 3 (Stretch goals)

| From | To | Notes |
|---|---|---|
| PDF | Plain text | PDF text extraction via `lopdf` — fidelity varies significantly by PDF source |
| Images | PDF | Embed images into a PDF via `lopdf` |
| PPTX | PDF or images | Complex — evaluate available Rust crates at the time |
| Audio format conversions | - | Possible via `symphonia` (pure Rust audio) but large binary |
| Video format conversions | - | Likely too large for WASM; explicitly out of scope initially |

**Pandoc note:** Pandoc itself (Haskell) does not compile to WASM. There are WASM builds of it in experimental state but they are large (~30 MB+) and not production-ready. The Rust-native crate approach gives better control over binary size and conversion quality per format.

---

## 8. Frontend Architecture

### Technology Choices

**No heavyweight JS framework.** React/Vue/Svelte add complexity and bundle size for what is fundamentally a simple single-page UI. The frontend is written in TypeScript using Web Components and the browser's native APIs. This keeps the JS bundle under 50 KB and reduces attack surface.

**esbuild** for bundling. It's extremely fast and produces clean output. No webpack config hell.

**CSS custom properties** for theming. No CSS framework dependency. A small, handcrafted design system using `--color-*`, `--space-*`, and `--radius-*` tokens. Dark mode via `prefers-color-scheme` media query.

### UI Structure

**Landing page / app shell:**
- Clean drag-and-drop zone, dominant in the viewport
- Format selector (auto-detected from dropped file, or manually overridden)
- List of supported conversions on hover/focus
- Privacy badge: "Your files never leave this browser — verified" with a link to explanation page
- Usage counter in the corner: "3 / 10 free conversions today"

**Conversion flow:**
1. User drags file onto drop zone (or clicks to open file picker)
2. JS reads file into an `ArrayBuffer` using the File API — no XHR, no fetch
3. Usage quota is checked against the Worker (`/api/quota` — sends only a fingerprint, not the file)
4. If within quota: instantiate the appropriate WASM module (or use cached instance)
5. Pass `Uint8Array` of file bytes into the WASM `convert()` function
6. WASM returns converted bytes; progress callbacks update the UI during processing
7. JS creates a `Blob` and triggers a synthetic `<a download>` click — file downloads locally
8. Quota count is incremented server-side

**Paywall:**
- On the 11th conversion of the day, a modal appears offering upgrade
- Non-intrusive: first 10 conversions require no account whatsoever
- Upgrade options: "Day Pass" ($2), "Monthly" ($8), "Annual" ($60)
- Payment via Stripe Checkout (redirect to Stripe-hosted page, return to app)
- On return, JWT is issued by Worker, stored in `localStorage`, validated client-side before each conversion

**Privacy explainer page:**
A dedicated `/privacy-proof` page that explains exactly how the architecture works, includes annotated browser DevTools screenshots showing zero upload requests, and links to the open-source repository where anyone can audit the code. This is a marketing asset as much as a technical document.

### Accessibility

- Keyboard navigable drop zone (Space/Enter to trigger file picker)
- ARIA live regions for conversion progress announcements
- Sufficient colour contrast in both light and dark modes (WCAG AA minimum, targeting AAA)
- No animations by default; `prefers-reduced-motion` respected throughout

### Offline Support

A service worker (compiled from Rust via `workers-rs` or handwritten JS if Rust SW support is insufficient) pre-caches:
- The app shell HTML/CSS/JS
- All WASM binaries that the user has previously loaded

After first visit, the app works with no network connection. The only thing that won't work offline is the quota check Worker call — the app degrades gracefully by showing a "you're offline, conversion limit not verified" message and still allowing conversion (no gating on offline).

---

## 9. Cloudflare Workers — Edge Backend

All Workers are written in Rust using `workers-rs`. This keeps the entire codebase in one language, enables shared types between the Worker and WASM crates, and avoids JS runtime overhead.

### `GET /api/quota`

Request: `{ fingerprint: string }` (no file data, ever)

The fingerprint is a SHA-256 hash of browser characteristics (User-Agent, timezone, screen resolution, language) computed client-side in JS. It's not a robust tracker — it's just a soft rate-limit signal. Privacy-conscious users who object to fingerprinting can create an account instead.

Response: `{ used: number, limit: number, reset_at: string }`

Worker logic:
1. Validate fingerprint is well-formed (prevent abuse of KV key namespace)
2. KV get `quota:{fingerprint}` — get current count + TTL
3. Return count and time until daily reset

### `POST /api/quota/increment`

Called after a successful conversion (not before — we don't charge the quota for failed conversions).

Request: `{ fingerprint: string }` or `{ jwt: string }` (if logged in)

Worker logic:
1. For fingerprint users: KV write `quota:{fingerprint}` = count + 1, TTL = seconds until midnight UTC
2. For JWT users: validate JWT signature, extract tier, check if unlimited, if not increment in KV

### `POST /api/license/issue`

Called by the Stripe/Paddle webhook after a successful payment.

Request (from payment provider webhook, HMAC-validated): payment confirmation + customer email + plan type

Worker logic:
1. Validate webhook signature (HMAC-SHA256 against shared secret stored in Worker secrets)
2. Generate a signed JWT using a private key stored as a Worker secret (RS256)
3. JWT payload: `{ sub: email_hash, tier: "pro", exp: unix_timestamp, conversions_remaining: null (unlimited) | number }`
4. Store JWT → plan mapping in KV for revocation support
5. Return JWT to be emailed to customer (or redirect URL with JWT fragment)

### `POST /api/license/validate`

Lightweight endpoint the client calls to confirm a stored JWT is still valid (not revoked).

### Worker Security Hardening

- All Worker routes enforce CORS headers permitting only the production domain
- Rate limiting on quota endpoints (Cloudflare's native rate limiting binding) to prevent KV abuse
- Input validation on all parameters before any KV access
- JWT signing keys rotated quarterly using Cloudflare Workers Secrets versioning
- Webhook endpoints validate HMAC before any processing

---

## 10. Monetisation & Licensing Layer

### Free Tier

- 10 conversions per day
- No account required
- Full feature access (no format restrictions — paywalling formats is a bad UX)
- Rate limited by browser fingerprint

### Paid Tiers

| Plan | Price | Conversions | Account |
|---|---|---|---|
| Day Pass | $2 | Unlimited for 24h | No account needed — JWT via email |
| Pro Monthly | $8/month | Unlimited | Email account |
| Pro Annual | $60/year ($5/month) | Unlimited | Email account |
| Team (future) | $20/month per seat | Unlimited | SSO / team management |

### Payment Flow

Using **Paddle** (recommended over Stripe for a solo/small team): Paddle acts as the merchant of record, handling EU VAT, sales tax, and international compliance automatically. This avoids the significant compliance overhead of handling tax yourself.

Flow:
1. User hits limit → paywall modal shown
2. User selects plan → redirected to Paddle Checkout (hosted, no card data touches our infrastructure)
3. Paddle sends webhook to `/api/license/issue` on payment success
4. Worker generates JWT, stores in KV, returns to Paddle as fulfillment data
5. Paddle emails the JWT (or a redemption link) to the customer
6. User pastes JWT into the app (or clicks redemption link which sets it automatically) — stored in `localStorage`
7. From that point, every conversion validates the JWT locally (signature check) before calling the quota endpoint

**Why JWT stored locally and validated edge-side?**
The client checks the JWT signature locally (public key embedded in the JS bundle) to avoid a round-trip on every conversion. The edge Worker does a secondary check before incrementing KV. The public key being in the JS bundle is fine — it's meant to be public.

### Conversion Count for Paid Users

Paid users get unlimited conversions. The "limit" concept disappears entirely — no counter shown, no warnings. This is deliberate: the moment a paying user sees a counter or a warning, it creates anxiety and erodes goodwill. Paid means unrestricted.

---

## 11. Security Model

### What the Server Never Sees

- File contents (any format, any size)
- File names
- File sizes
- Conversion parameters (format choices, quality settings)

The Worker receives only: a fingerprint or JWT, and an increment signal. That's all.

### What the Server Does See

- Source IP (Cloudflare infrastructure sees this always — unavoidable)
- A browser fingerprint hash (not reversible to identity by design)
- Number of conversions (count only, no timing or pattern data stored beyond TTL)
- Payment information (via Paddle — handled entirely by Paddle, not us)
- JWT claims (email hash, tier, expiry — no PII in the JWT itself)

### Trust Model Documentation

A public `SECURITY.md` and a dedicated `/how-it-works` page explain exactly what data the server receives. The source code is open (at minimum the conversion crate and the Worker code) so the claims are auditable. This transparency is a core feature, not an afterthought.

### Content Security Policy

Strict CSP headers served from Cloudflare Pages:
```
Content-Security-Policy:
  default-src 'none';
  script-src 'self';
  style-src 'self';
  connect-src 'self' https://api.paddle.com;
  worker-src 'self';
  wasm-src 'self';
  img-src 'self' data:;
  frame-src https://checkout.paddle.com;
```

The `wasm-src` directive whitelists WASM execution. The `connect-src` only allows calls to our own Worker and Paddle's checkout API. Notably absent: any analytics, CDN, or third-party script domains.

### Dependency Auditing

`cargo audit` is run in CI on every push. Any advisory against a dependency in the tree fails the build. Dependencies are updated on a monthly schedule or immediately for security advisories.

---

## 12. Performance Strategy

### Initial Page Load Target: < 2 seconds on 4G

- App shell HTML + CSS + JS: < 50 KB (no framework)
- No WASM loaded on initial paint — lazy loaded only when user drops a file
- Critical CSS inlined in `<head>`; full CSS loaded async
- Service worker pre-caches on first visit so second visit is near-instant

### Conversion Performance

WASM runs at 60–90% of native Rust speed in modern browsers. For reference benchmarks:
- PNG → JPEG (4K image): target < 200ms
- Markdown → HTML (100-page document): target < 50ms
- CSV → JSON (50,000 rows): target < 300ms
- DOCX → plain text (200-page document): target < 500ms

Large files (>50 MB) are processed in chunks using WASM's memory stream capabilities where the conversion crate supports it, with progress reported back to the UI to prevent the appearance of a frozen browser.

### Web Workers for Conversion

All WASM execution happens in a dedicated Web Worker thread (separate from the main thread). This ensures the UI remains responsive during conversion — the drop zone, progress bar, and cancel button all stay interactive. The WASM module is instantiated inside the Web Worker context.

This requires `SharedArrayBuffer` for zero-copy data transfer, which requires:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```
These headers are set in Cloudflare Pages' `_headers` file. This is a deliberate architectural choice — COEP/COOP is the right way to enable `SharedArrayBuffer` and also improves isolation against Spectre-class attacks.

---

## 13. Testing Strategy

### Unit Tests (Rust)

Every conversion function has unit tests in `crates/converter/src/`. Tests run natively (not in WASM) using `cargo nextest` for speed. The conversion logic is pure functions (bytes in, bytes out) so testing is straightforward.

### WASM Integration Tests

`wasm-pack test --node` runs tests in a headless Node.js environment using the WASM target. These verify the JS/WASM binding layer works correctly — that the `wasm-bindgen` API surface behaves as expected.

### Golden File Tests

A test suite in `scripts/test-conversions.sh` takes fixture files from `tests/fixtures/`, runs each conversion, and compares the output against golden files in `tests/golden/`. Mismatches fail CI. Golden files are committed and updated deliberately (any change to a golden file requires an explicit commit explaining why the output changed).

### End-to-End Tests (Playwright)

A small Playwright suite tests the critical user path in a real browser:
- Drop a file, verify conversion, verify download, verify no network requests to non-allowed domains
- Quota increment behaviour
- Paywall modal appears at limit
- JWT-based access restores unlimited conversions

The Playwright tests run in CI against the preview deployment URL (not localhost) to test the real Cloudflare infrastructure.

### Performance Regression Tests

A dedicated CI step uses `hyperfine` (or a custom Rust benchmarking script) to measure WASM conversion time against fixture files. If any conversion is more than 20% slower than the previous baseline, CI fails and forces a review.

---

## 14. Observability & Analytics

### What We Measure

Conversion events are counted (not logged — just incremented counters) via Cloudflare Workers Analytics Engine:
- Conversions by format pair (e.g. "png→jpeg: 1,203 today")
- Quota hit rate (how often free users hit the limit)
- Error rate by conversion type
- WASM load time (reported by client after first load)

Crucially, **no file metadata is in these events**. The analytics events contain only: conversion type, success/failure, timestamp bucket (hourly), and tier (free/paid). No IP, no fingerprint, no user identifier.

### What We Explicitly Do Not Measure

- File contents
- File names or sizes
- Individual user sessions
- Geographic location beyond region (Cloudflare provides this in aggregate)

This privacy-first analytics approach is documented publicly and is part of the brand promise.

### Error Monitoring

Errors in the Worker are captured by Cloudflare's built-in error logging. WASM panics in production are caught and surfaced as user-facing error messages (via `console_error_panic_hook` in debug, controlled error responses in release).

### No Third-Party Analytics

No Google Analytics, Plausible, Fathom, or any external analytics script. This is non-negotiable — a privacy-first product that loads third-party tracking scripts is a contradiction that technically literate users will notice immediately. All analytics are first-party via Cloudflare.

---

## 15. Domain, Branding & Launch

### Name

**Transfigure** — private-first file conversion that runs entirely in your browser. Domain: **transfigure.ing** (registered, £7/yr).

### Domain & DNS

Register via Cloudflare Registrar (no markup on wholesale pricing, DNSSEC automatic). Cloudflare manages DNS, so Pages and Workers integrate without any manual DNS configuration.

### Launch Strategy

1. **GitHub public repo** from day one — the open-source nature is a trust signal and marketing asset
2. **Hacker News Show HN post** — "Show HN: File converter that runs entirely in your browser (Rust/WASM)" — this audience cares deeply about privacy and will appreciate the technical approach
3. **Product Hunt launch** — target a Tuesday/Wednesday launch, prepare a detailed maker post explaining the privacy architecture
4. **Reddit posts** in r/privacy, r/rust, r/webdev, r/selfhosted
5. **Dev.to / Hashnode article** explaining the WASM architecture — drives developer traffic and backlinks

### SEO Strategy

Static HTML for the landing page and format-specific pages (e.g. `/convert/png-to-jpeg`, `/convert/markdown-to-html`) with proper meta tags. These pages describe the privacy-preserving approach for that specific conversion and target long-tail search terms. Each page is pre-rendered at build time (no JS required for the informational content).

---

## 16. Phased Delivery Plan

### Phase 0 — Foundation (Weeks 1–2)
- [x] Initialise monorepo with workspace `Cargo.toml`
- [x] Set up devcontainer with all tooling
- [x] Create GitHub repository, configure branch protection
- [x] Set up Cloudflare Pages project, link GitHub repo
- [x] Implement `ci.yml` and `deploy.yml` GitHub Actions
- [x] Deploy "Hello World" static page end-to-end through the full pipeline
- [x] Worker skeleton: `/api/quota` returns mock data

### Phase 1 — Core Conversion Engine (Weeks 3–6)
- [ ] Image conversion (PNG ↔ JPEG ↔ WebP ↔ BMP) in WASM
- [ ] Markdown → HTML, Markdown → text in WASM
- [ ] CSV ↔ JSON ↔ TSV in WASM
- [ ] ZIP creation from multiple files
- [ ] WASM module lazy loading infrastructure in JS
- [ ] Web Worker offload for WASM execution
- [ ] Basic drag-and-drop UI with progress feedback
- [ ] Download trigger from converted bytes
- [ ] Golden file test suite for all implemented conversions

### Phase 2 — Quota & Infrastructure (Weeks 7–9)
- [ ] Real quota Worker with KV persistence
- [ ] Client fingerprinting (privacy-respecting, explained to user)
- [ ] Quota display in UI ("3 / 10 free conversions today")
- [ ] Paywall modal at limit
- [ ] Paddle integration (webhook → JWT issuance)
- [ ] JWT validation in Worker and client
- [ ] Service worker for offline caching
- [ ] COEP/COOP headers, SharedArrayBuffer enabled

### Phase 3 — Office Formats (Weeks 10–14)
- [ ] DOCX → plain text and Markdown
- [ ] Markdown → DOCX
- [ ] XLSX/XLS → CSV and JSON
- [ ] CSV → XLSX
- [ ] JSON ↔ YAML ↔ TOML
- [ ] SVG → PNG via `resvg`
- [ ] Evaluate PDF text extraction viability

### Phase 4 — Polish & Launch (Weeks 15–16)
- [ ] Privacy explainer page with DevTools screenshots
- [ ] `/how-it-works` technical deep-dive page
- [ ] SEO pages for each conversion type
- [ ] Accessibility audit and fixes
- [ ] Performance regression test suite locked in
- [ ] Security review of CSP, Worker inputs, JWT implementation
- [ ] Public launch (HN, Product Hunt, Reddit)

### Phase 5 — Post-Launch Iteration (Ongoing)
- [ ] Usage analytics review — what conversions are most popular?
- [ ] Add most-requested formats based on real data
- [ ] Batch conversion UI (multiple files, multiple formats in one operation)
- [ ] Team plan with shared JWT management
- [ ] API product: same WASM engine available as an npm package for developers

---

## 17. Cost Model Summary

| Component | Free Tier | At 1k DAU | At 10k DAU |
|---|---|---|---|
| Cloudflare Pages (hosting) | $0 | $0 | $0 |
| Cloudflare Workers | $0 (100k req/day) | $5/mo | $5/mo |
| Workers KV | $0 (1k writes/day) | $5/mo (included) | ~$15/mo |
| Paddle fees | 5% + $0.50/transaction | Variable | Variable |
| Domain | ~$10/year | $10/year | $10/year |
| **Total infrastructure** | **$0** | **~$10/mo** | **~$20/mo** |

Infrastructure cost is effectively zero until significant traffic. Revenue from even a handful of paying users covers all costs comfortably.

---

## 18. Open Questions & Decisions

These need to be decided before or early in implementation:

1. ~~**Final product name and domain.**~~ Decided: **Transfigure** at `transfigure.ing`.

2. **PDF conversion scope.** PDF text extraction is feasible but fidelity varies enormously based on how the PDF was created. Set clear user expectations or restrict to "PDF created from text-based sources." PDF *generation* (image → PDF, markdown → PDF) is more reliably achievable.

3. **Account system.** The plan avoids user accounts entirely for free users. For paid users, JWT-via-email is lightweight but means losing access if email is lost. Decide whether a lightweight account system (just email + hashed password in KV, no PII) is worth the added complexity for paid tier.

4. **Paddle vs Stripe.** Paddle handles VAT/sales tax as merchant of record (significant compliance advantage for solo operation). Stripe is more widely known and has better developer tooling. If you're comfortable handling tax compliance yourself, Stripe is simpler. For a bootstrapped product, Paddle is the lower-risk choice.

5. **Open source model.** Fully open source (MIT/Apache 2.0) maximises trust and community contribution but means competitors can self-host. A possible middle ground: open source the conversion crate (the privacy-relevant part) while keeping the Worker and payment integration closed. Decision affects launch positioning.

6. **Service worker in Rust or JS.** `workers-rs` is designed for Cloudflare Workers, not browser Service Workers. A browser SW in vanilla JS is simpler and more mature. The all-Rust goal may need to yield here — a 100-line JS service worker is not a meaningful compromise.

7. **WASM module splitting granularity.** Fine-grained splitting (one module per format pair) minimises initial load but adds complexity to the loading orchestration. Coarser splitting (by category: images, documents, office) is simpler. The right answer depends on measured binary sizes during Phase 1.

---

*Document version: 1.0 | Status: Pre-implementation planning | Last updated: March 2026*