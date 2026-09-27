# ⚗️ Transfigure

> Private-first file conversion. Everything runs in your browser. Your files never leave your machine.

[![Deploy](https://img.shields.io/github/actions/workflow/status/RephlexZero/transfigure/deploy.yml?label=deploy)](https://github.com/RephlexZero/transfigure/actions)
[![License](https://img.shields.io/github/license/RephlexZero/transfigure)](LICENSE)

---

## What it is

Transfigure is a file converter that runs entirely inside the browser as a compiled [WebAssembly](https://webassembly.org/) binary. Drag a file on, get a converted file back — zero bytes ever leave your device.

This is a structural privacy guarantee, not a policy one. There is no server receiving your files. You can verify it yourself in browser DevTools: no upload network requests are made during conversion.

**Works offline.** Once the WASM binary is cached by the browser, the app functions with no internet connection at all.

---

## Supported conversions

| Category        | Input                                              | Output                                         |
|-----------------|----------------------------------------------------|------------------------------------------------|
| Images          | HEIC/HEIF, JPG, PNG, WebP, GIF, BMP, TIFF, ICO, SVG, QOI, TGA, DDS (BC1–BC7), HDR, EXR | JPG, PNG, WebP, AVIF, PDF, GIF, BMP, TIFF, ICO, QOI, TGA |
| Audio           | MP3, M4A (AAC/ALAC), AAC, WAV, FLAC, OGG, AIFF, CAF | MP3, WAV, FLAC                                 |
| Documents       | DOCX, ODT, RTF, PDF (text), Markdown, HTML, TXT    | PDF, DOCX, HTML, Markdown, TXT                 |
| Spreadsheets    | XLSX, XLS, ODS, CSV, TSV, JSON records             | XLSX, CSV, JSON, TSV                           |
| Structured data | JSON, YAML, TOML, XML                              | JSON, YAML, TOML, XML                          |
| Encoding        | Any file / Base64                                  | Base64 / original bytes                        |

Documents go through one shared model (headings, emphasis, links, lists, quotes, code, tables), so every document input converts to every document output. JPEG and HEIC photos are rotated according to their EXIF orientation. Lossless audio keeps its bit depth; the quality slider sets JPG/AVIF quality and MP3 bitrate.

Batch conversion is supported — drop multiple files at once and convert them all in a single click. Converted files can be downloaded individually or packaged as a ZIP, tar.gz, tar.xz or 7z archive (duplicate names get numbered).

---

## Tech stack

| Layer      | Technology                                                   |
|------------|--------------------------------------------------------------|
| UI         | [Leptos](https://leptos.dev/) (Rust → WASM, CSR)            |
| Styling    | [Tailwind CSS v3](https://tailwindcss.com/), self-hosted Inter + JetBrains Mono |
| Build      | [Trunk](https://trunkrs.dev/)                                |
| Conversion | Pure Rust compiled to WebAssembly via `wasm-bindgen`         |
| Images     | [`image`](https://github.com/image-rs/image), [`heic`](https://github.com/imazen/heic) (HEIC), [`resvg`](https://github.com/linebender/resvg) (SVG), [`bcdec_rs`](https://crates.io/crates/bcdec_rs) (DDS) |
| Audio      | [`symphonia`](https://github.com/pdeljanov/Symphonia) (decode), [`hound`](https://github.com/ruuda/hound) (WAV), [`flacenc`](https://github.com/yotarok/flacenc-rs) (FLAC), [`rusty_mp3`](https://crates.io/crates/rusty_mp3) (MP3) |
| Documents  | [`comrak`](https://github.com/kivikakk/comrak), [`htmd`](https://github.com/letmutex/htmd), [`docx-rs`](https://github.com/bokuweb/docx-rs), [`lopdf`](https://github.com/J-F-Liu/lopdf), and a built-in PDF layout engine |
| Data       | [`calamine`](https://github.com/tafia/calamine), [`rust_xlsxwriter`](https://github.com/jmcnamara/rust_xlsxwriter), [`serde_yaml_ng`](https://github.com/acatton/serde-yaml-ng), [`toml`](https://github.com/toml-rs/toml), [`quick-xml`](https://github.com/tafia/quick-xml) |
| Hosting    | [Cloudflare Pages](https://pages.cloudflare.com/)            |
| CI/CD      | GitHub Actions                                               |

### Workspace layout

```
transfigure/
├── crates/
│   ├── app/                # Leptos frontend (compiled to WASM)
│   │   ├── src/components/ # UI components (header, hero, converter, info, icons)
│   │   ├── fonts/          # Self-hosted variable fonts (OFL)
│   │   ├── src/utils.rs    # Download/format helpers
│   │   └── index.html      # Entry point for Trunk
│   └── converter/      # Conversion engine (no WASM dependencies)
│       ├── src/
│       │   ├── lib.rs          # Public API: convert(), get_output_formats()
│       │   ├── image_conv.rs   # Images (image crate, HEIC, SVG, ICO, image → PDF)
│       │   ├── dds.rs          # DDS textures (BC1–BC7, uncompressed)
│       │   ├── audio.rs        # Decode (symphonia), encode WAV / FLAC / MP3
│       │   ├── doc/            # Document model + readers and writers per format
│       │   ├── pdf.rs          # PDF writer: text layout and image pages
│       │   ├── table.rs        # CSV / TSV / XLSX / XLS / ODS / JSON records
│       │   ├── structured.rs   # JSON / YAML / TOML / XML
│       │   └── archive.rs      # ZIP, tar.gz, tar.xz, 7z output
│       ├── tests/matrix.rs     # Runs every offered conversion on real fixtures
│       └── tests/fixtures/     # Fixtures made by ffmpeg, LibreOffice, Pillow (generate.py)
├── input.css           # Tailwind source
├── Trunk.toml          # Trunk build config
├── Cargo.toml          # Workspace manifest
└── package.json        # Tailwind build scripts
```

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain + `wasm32-unknown-unknown` target)
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`
- [Node.js](https://nodejs.org/) (for Tailwind CSS)

```sh
rustup target add wasm32-unknown-unknown
npm install
```

### Run locally

```sh
trunk serve
```

Trunk will:
1. Compile the Leptos app to WASM
2. Run Tailwind CSS before each build (via the pre-build hook in `Trunk.toml`)
3. Serve the app at `http://localhost:8080` with hot-reload on changes to `crates/`

### Build for production

```sh
trunk build --release
```

Output is written to `dist/`. The `--release` profile applies `opt-level = "z"`, LTO, and stripping to minimise the WASM binary size.

### Test the converter

The converter crate has no WASM dependencies and is tested natively. `tests/matrix.rs` runs every conversion the UI offers against fixtures produced by other tools and checks each output decodes correctly:

```sh
cargo test -p converter
# Keep every output for inspection with external tools:
TRANSFIGURE_DUMP=/tmp/out cargo test -p converter --test matrix
# Try a single conversion:
cargo run -p converter --example convert -- photo.heic heic jpg photo.jpg
```

Fixtures are regenerated with [uv](https://docs.astral.sh/uv/) (Python tooling is linted with `ruff` and type-checked with `ty`):

```sh
uv run crates/converter/tests/fixtures/generate.py
```

---

## How conversion works

1. The user drops files onto the drop zone (or clicks to browse).
2. Each file's bytes are read into memory via the [File API](https://developer.mozilla.org/en-US/docs/Web/API/File).
3. The Leptos app calls `converter::convert(&bytes, config_json)` — a pure Rust function compiled into the same WASM binary.
4. The output bytes are handed back to the browser, which constructs a temporary object URL and triggers a download. No network request is involved.

---

## Contributing

1. Fork the repository and create a feature branch.
2. Run `cargo clippy --all-targets` and `cargo fmt --all` before committing.
3. Open a pull request — the CI workflow will run format/lint/test checks automatically.

---

## License

Licensed under GNU Affero General Public License v3.0 only (AGPL-3.0-only). See [LICENSE](LICENSE).
