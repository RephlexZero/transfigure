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

| Category   | Input               | Output formats                          |
|------------|---------------------|-----------------------------------------|
| Images     | PNG                 | JPG, WebP, GIF, BMP, TIFF               |
|            | JPG / JPEG          | PNG, WebP, GIF, BMP, TIFF               |
|            | WebP                | PNG, JPG, GIF, BMP, TIFF                |
|            | GIF                 | PNG, JPG, WebP, BMP, TIFF               |
|            | BMP                 | PNG, JPG, WebP, GIF, TIFF               |
|            | TIFF / TIF          | PNG, JPG, WebP, GIF, BMP                |
|            | SVG                 | PNG                                     |
| Documents  | Markdown (MD)       | HTML, TXT                               |
|            | HTML                | Markdown                                |
| Data       | CSV                 | JSON, TSV                               |
|            | TSV                 | CSV                                     |
|            | JSON                | CSV, YAML, TOML                         |
|            | YAML / YML          | JSON                                    |
|            | TOML                | JSON                                    |
| Encoding   | Base64              | Binary (decoded)                        |
|            | Any file            | Base64 (encoded)                        |

Batch conversion is supported — drop multiple files at once and convert them all in a single click. Converted files can be downloaded individually or packaged as a ZIP, tar.gz, or tar.xz archive.

---

## Tech stack

| Layer      | Technology                                                   |
|------------|--------------------------------------------------------------|
| UI         | [Leptos](https://leptos.dev/) (Rust → WASM, CSR)            |
| Styling    | [Tailwind CSS v3](https://tailwindcss.com/) + [DaisyUI v4](https://daisyui.com/) |
| Build      | [Trunk](https://trunkrs.dev/)                                |
| Conversion | Pure Rust compiled to WebAssembly via `wasm-bindgen`         |
| Hosting    | [Cloudflare Pages](https://pages.cloudflare.com/)            |
| CI/CD      | GitHub Actions                                               |

### Workspace layout

```
transfigure/
├── crates/
│   ├── app/            # Leptos frontend (compiled to WASM)
│   │   ├── src/main.rs # All UI components
│   │   └── index.html  # Entry point for Trunk
│   └── converter/      # Conversion engine (no WASM dependencies)
│       └── src/
│           ├── lib.rs          # Public API: convert(), get_output_formats()
│           ├── image_conv.rs   # Image conversions via the `image` crate + resvg
│           ├── document.rs     # Markdown, HTML, YAML, TOML, Base64
│           ├── spreadsheet.rs  # CSV / TSV / JSON
│           └── archive.rs      # ZIP, tar.gz, tar.xz output
├── input.css           # Tailwind source
├── Trunk.toml          # Trunk build config
├── Cargo.toml          # Workspace manifest
└── package.json        # Tailwind/DaisyUI build scripts
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

### Check the converter crate

The converter crate has no WASM dependencies and can be checked/tested on the native toolchain:

```sh
cargo check -p converter
cargo test -p converter
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

See [LICENSE](LICENSE).
