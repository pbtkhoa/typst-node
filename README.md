# typst-node

[![CI](https://github.com/pbtkhoa/typst-node/actions/workflows/CI.yml/badge.svg)](https://github.com/pbtkhoa/typst-node/actions/workflows/CI.yml)
[![npm version](https://img.shields.io/npm/v/typst-node.svg)](https://www.npmjs.com/package/typst-node)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Node.js](https://img.shields.io/badge/Node.js-18%2B-green.svg)](https://nodejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

Fast, zero-disk-I/O Node.js native addon for [Typst](https://typst.app/) compilation and rendering (PDF, SVG, PNG).

Built with Rust and [napi-rs](https://napi.rs/) for serverless functions, containers, and high-throughput document services.

---

## ✨ Features

- ⚡ **Fast:** ~11ms P50 latency for invoices, >220 req/s on 8 cores (10–15× faster than Chromium/Puppeteer).
- 💾 **Zero Disk I/O:** Compiles in memory via `Buffer`. No temp files, safe for AWS Lambda and Docker.
- 🧵 **Non-Blocking:** Typesetting runs on libuv worker threads without blocking the Node.js event loop.
- 📄 **Multi-Format:** Export to multi-page PDF, SVG (per-page or merged), and high-DPI PNG.
- 🔤 **17 Embedded Fonts:** Ships with standard fonts (_Libertinus Serif_, _DejaVu Sans Mono_). No system font setup required.
- 🔄 **Dynamic Data:** Pass JavaScript objects directly into templates via `#sys.inputs`.
- 🗂️ **Virtual Assets:** Map images and sub-templates in memory for `#image()` and `#include`.
- 🌊 **Node.js Streaming:** Native `stream.Readable` support for chunked PDF bytes and page-by-page PNG/SVG.
- 🧠 **AST Memoization:** Thread-safe template cache for $O(1)$ syntax tree reuse across repeated compiles.
- 🛠️ **Prebuilt Binaries:** No `node-gyp` or Rust compiler needed on install.

---

## 🖥️ Supported Platforms

| Platform                 | Arch                    | Binary Package                                             |
| ------------------------ | ----------------------- | ---------------------------------------------------------- |
| 🍎 macOS                 | Apple Silicon (`arm64`) | `typst-node-darwin-arm64`                                  |
| 🍎 macOS                 | Intel (`x64`)           | `typst-node-darwin-x64`                                    |
| 🐧 Linux (glibc)         | x64 / arm64             | `typst-node-linux-x64-gnu`, `typst-node-linux-arm64-gnu`   |
| 🏔️ Linux (musl / Alpine) | x64 / arm64             | `typst-node-linux-x64-musl`, `typst-node-linux-arm64-musl` |
| 🪟 Windows               | x64                     | `typst-node-win32-x64-msvc`                                |

---

## 📦 Installation

```bash
npm i typst-node
# yarn add typst-node
# pnpm add typst-node
```

---

## 🚀 Quick Start

### 1. 📄 Render PDF

```typescript
import { TypstRenderer } from 'typst-node'
import * as fs from 'node:fs/promises'

const renderer = new TypstRenderer()

const pdf = await renderer.renderPdf({
  template: `
    #set page(paper: "a4", margin: 2cm)
    #set text(font: "Libertinus Serif", size: 12pt)

    = Hello from typst-node
    Compiled entirely in memory.
  `,
})

await fs.writeFile('output.pdf', pdf)
```

### 2. 🔄 Dynamic Data Injection (`#sys.inputs`)

Pass any JSON-serializable object to `data`. Access it in Typst via `#sys.inputs`:

```typescript
const pdf = await renderer.renderPdf({
  template: `
    #set page(paper: "a4", margin: 2cm)

    #align(right)[#text(size: 18pt, weight: "bold")[INVOICE]]

    *Billed To:* #sys.inputs.client \\
    *Invoice #:* #sys.inputs.invoice_number

    #table(
      columns: (3fr, 1fr, 1fr),
      [*Item*], [*Qty*], [*Price*],
      ..sys.inputs.items.map(it => (it.name, str(it.qty), "$" + str(it.price))).flatten()
    )

    #align(right)[*Total:* $#sys.inputs.items.map(it => it.qty * it.price).sum()]
  `,
  data: {
    client: 'Acme Corp',
    invoice_number: 'INV-1001',
    items: [
      { name: 'Cloud Server', qty: 2, price: 50 },
      { name: 'Support Plan', qty: 1, price: 100 },
    ],
  },
})
```

### 3. 🎨 PNG & SVG Export

```typescript
const template = '= Page 1\n#pagebreak()\n= Page 2'

// High-DPI PNG (144 DPI by default)
const [page1Png] = await renderer.renderPng({
  template,
  ppi: 144, // 72 for web, 288 for print
  pages: [1], // Optional 1-indexed page filter
})

// SVG vector strings
const [page1Svg, page2Svg] = await renderer.renderSvg({ template })

// Merged multi-page SVG
const mergedSvg = await renderer.renderSvgMerged({ template })
```

### 4. 🗂️ In-Memory Virtual Assets & Images

Resolve `#image` and `#include` from memory without touching the filesystem:

```typescript
const pdf = await renderer.renderPdf({
  template: `
    #include "header.typ"
    #image("logo.png", width: 30mm)
    = Monthly Report
  `,
  assets: {
    'logo.png': await fs.readFile('./logo.png'),
    'header.typ': Buffer.from('== ACME Global Inc.'),
  },
})
```

### 5. 🌊 Node.js Streaming (`stream.Readable`)

Stream binary chunks or pages directly to HTTP responses or disk:

```typescript
// Stream PDF in 64KB chunks
const stream = renderer.renderPdfStream({ template: '= Large Document\n...' }, 64 * 1024);
stream.pipe(fs.createWriteStream('doc.pdf'));

// Stream rendered PNG pages (object mode)
const pngStream = renderer.renderPngStream({ template: '...' });
for await (const { page, data } of pngStream) {
  console.log(`Page ${page}: ${data.length} bytes`);
}
```

### 6. 🔤 Custom Fonts & AST Cache

```typescript
const renderer = new TypstRenderer({
  fontPaths: ['./fonts'], // Custom font directories
  includeSystemFonts: true, // Scan OS fonts (default: false)
  enableCache: true, // AST memoization (default: true)
  cacheCapacity: 1024, // Max cached templates (default: 1024)
})

// Cache management
console.log('Cached AST count:', renderer.cacheSize())
renderer.clearCache()
```

---

## ⚡ Benchmark

Tested on Apple M2 (8 cores, Node.js v26.13):

### ⏱️ Latency vs Alternatives

| Scenario       | typst-node (P99) | PDFKit  | Puppeteer (Chromium) |
| -------------- | ---------------- | ------- | -------------------- |
| 1-page Receipt | **12.28 ms**     | ~34 ms  | ~210 ms              |
| 3-page Invoice | **17.24 ms**     | ~48 ms  | ~260 ms              |
| 20-page Report | **58.60 ms**     | ~180 ms | ~750 ms              |

### 🚦 Concurrency Scaling

Non-blocking worker threads handle high concurrency with near-zero event loop delay:

| Concurrency  | Throughput      | Avg Latency | Event Loop Lag |
| ------------ | --------------- | ----------- | -------------- |
| 10 requests  | 196.5 req/s     | 5.09 ms     | **1.33 ms**    |
| 50 requests  | 224.7 req/s     | 4.45 ms     | **0.79 ms**    |
| 100 requests | **224.3 req/s** | 4.46 ms     | **1.15 ms**    |

---

## 📖 API Reference

### `JsTypstRendererOptions`

| Option               | Type       | Default | Description                     |
| -------------------- | ---------- | ------- | ------------------------------- |
| `fontPaths`          | `string[]` | `[]`    | Extra font folder paths to scan |
| `includeSystemFonts` | `boolean`  | `false` | Scan system-installed fonts     |
| `enableCache`        | `boolean`  | `true`  | Enable template AST memoization |
| `cacheCapacity`      | `number`   | `1024`  | Maximum cached templates        |

### `TypstRenderer` Methods

| Method                                 | Returns             | Description                                  |
| -------------------------------------- | ------------------- | -------------------------------------------- |
| `renderPdf(options)`                   | `Promise<Buffer>`   | Compiles document to PDF buffer              |
| `renderPng(options)`                   | `Promise<Buffer[]>` | Rasterizes pages to PNG buffers              |
| `renderSvg(options)`                   | `Promise<string[]>` | Renders pages as SVG strings                 |
| `renderSvgMerged(options)`             | `Promise<string>`   | Renders multi-page merged SVG                |
| `renderPdfStream(options, chunkSize?)` | `Readable`          | Streams PDF in binary chunk buffers          |
| `renderPngStream(options)`             | `Readable`          | Object-mode stream emitting `{ page, data }` |
| `renderSvgStream(options)`             | `Readable`          | Object-mode stream emitting `{ page, svg }`  |
| `cacheSize()`                          | `number`            | Current number of cached template ASTs       |
| `clearCache()`                         | `void`              | Clears all cached template ASTs              |

---

## 📄 License

[MIT](./LICENSE) © [Khoa Pham](https://github.com/pbtkhoa)
