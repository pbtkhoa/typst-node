import test from 'ava'
import { TypstRenderer } from '../index'

test('TypstRenderer initializes with embedded fonts', (t) => {
  const renderer = new TypstRenderer()
  t.truthy(renderer)
})

test('TypstRenderer renders PDF buffer asynchronously', async (t) => {
  const renderer = new TypstRenderer()
  const pdfBuffer = await renderer.render({
    template: '= Hello Typst from Node.js\nThis is rendered via background libuv worker thread.',
  })

  t.true(Buffer.isBuffer(pdfBuffer))
  t.true(pdfBuffer.length > 0)
  t.is(pdfBuffer.subarray(0, 5).toString('ascii'), '%PDF-')
})

test('TypstRenderer injects JSON dynamic data into sys.inputs', async (t) => {
  const renderer = new TypstRenderer()
  const pdfBuffer = await renderer.renderPdf({
    template: `
#let title = sys.inputs.title
#let total = sys.inputs.items.map(it => it.price).sum()
= #title
Total price: #total
`,
    data: {
      title: 'Monthly Summary',
      items: [
        { name: 'Item A', price: 10 },
        { name: 'Item B', price: 25 },
      ],
    },
  })

  t.true(Buffer.isBuffer(pdfBuffer))
  t.is(pdfBuffer.subarray(0, 5).toString('ascii'), '%PDF-')
})

test('TypstRenderer renders SVG and merged SVG', async (t) => {
  const renderer = new TypstRenderer()
  const template = '= Page 1\n#pagebreak()\n= Page 2'

  const pages = await renderer.renderSvg({ template })
  t.is(pages.length, 2)
  t.true(pages[0].includes('<svg'))
  t.true(pages[1].includes('<svg'))

  const merged = await renderer.renderSvgMerged({ template })
  t.true(typeof merged === 'string')
  t.true(merged.includes('<svg'))
})

test('TypstRenderer renders PNG pages with custom ppi', async (t) => {
  const renderer = new TypstRenderer()
  const pngs = await renderer.renderPng({
    template: '= Test PNG Output',
    ppi: 144,
  })

  t.is(pngs.length, 1)
  t.true(Buffer.isBuffer(pngs[0]))
  // Check PNG magic bytes: 0x89 0x50 0x4e 0x47 0x0d 0x0a 0x1a 0x0a
  t.deepEqual(Array.from(pngs[0].subarray(0, 8)), [137, 80, 78, 71, 13, 10, 26, 10])
})

test('TypstRenderer loads in-memory assets for virtual includes', async (t) => {
  const renderer = new TypstRenderer()
  const pdfBuffer = await renderer.render({
    template: '= Main Document\n#include "sub.typ"',
    assets: {
      'sub.typ': Buffer.from('== Subheading\nIncluded from virtual in-memory buffer!'),
    },
  })

  t.true(Buffer.isBuffer(pdfBuffer))
  t.is(pdfBuffer.subarray(0, 5).toString('ascii'), '%PDF-')
})

test('TypstRenderer resolves nested relative virtual paths and in-memory image assets', async (t) => {
  const renderer = new TypstRenderer()

  // Minimal 1x1 valid PNG buffer
  const minimalPng = Buffer.from([
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49,
    0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00,
    0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
  ])

  const pdfBuffer = await renderer.renderPdf({
    template: `
#include "docs/intro.typ"
#image("./assets/logo.png")
#include "chapters/deep/section.typ"
`,
    assets: {
      // Key with leading slash
      '/assets/logo.png': minimalPng,
      // Key with relative ./ prefix
      './docs/intro.typ': Buffer.from('= Welcome to Docs\n#include "../shared/header.typ"'),
      'shared/header.typ': Buffer.from('== Shared Header Text'),
      // Deep nested include referencing parent relative image and root config
      'chapters/deep/section.typ': Buffer.from(`
=== Deep Section
#image("../../assets/logo.png")
#let cfg = json("/data/config.json")
Config name: #cfg.name
`),
      // Key with Windows-style backslash
      'data\\config.json': Buffer.from('{"name": "Typst Node Test"}'),
    },
  })

  t.true(Buffer.isBuffer(pdfBuffer))
  t.true(pdfBuffer.length > 0)
  t.is(pdfBuffer.subarray(0, 5).toString('ascii'), '%PDF-')
})

test('TypstRenderer throws clean diagnostic error on syntax error', async (t) => {
  const renderer = new TypstRenderer()
  const error = await t.throwsAsync(
    renderer.render({
      template: '= Title\n#non_existent_function_12345()',
    }),
  )

  t.truthy(error)
  t.true(error!.message.includes('non_existent_function_12345'))
  t.true(error!.message.includes('main.typ:2:'))
})

test('TypstRenderer handles concurrent compilations in parallel without blocking', async (t) => {
  const renderer = new TypstRenderer()
  const tasks = Array.from({ length: 10 }, (_, i) =>
    renderer.render({
      template: `= Concurrent Document #${i}\nRendered concurrently across libuv threads.`,
    }),
  )

  const results = await Promise.all(tasks)
  t.is(results.length, 10)
  for (const buf of results) {
    t.true(Buffer.isBuffer(buf))
    t.is(buf.subarray(0, 5).toString('ascii'), '%PDF-')
  }
})

test('TypstRenderer template AST memoization and cache management', async (t) => {
  const renderer = new TypstRenderer({ enableCache: true, cacheCapacity: 100 })
  t.is(renderer.cacheSize(), 0)

  const templateA = '= Template A\nHello #sys.inputs.name'
  await renderer.renderPdf({ template: templateA, data: { name: 'Alice' } })
  t.is(renderer.cacheSize(), 1)

  // Re-rendering same template with different dynamic data hits cache
  await renderer.renderPdf({ template: templateA, data: { name: 'Bob' } })
  t.is(renderer.cacheSize(), 1)

  // Different template increases cache size
  const templateB = '= Template B\nAnother page'
  await renderer.renderPdf({ template: templateB })
  t.is(renderer.cacheSize(), 2)

  // Clear cache resets entries to 0
  renderer.clearCache()
  t.is(renderer.cacheSize(), 0)

  // Test disabled cache
  const uncachedRenderer = new TypstRenderer({ enableCache: false })
  await uncachedRenderer.renderPdf({ template: templateA, data: { name: 'Charlie' } })
  t.is(uncachedRenderer.cacheSize(), 0)
})

test('TypstRenderer streams PDF in chunks via renderPdfStream', async (t) => {
  const renderer = new TypstRenderer()
  const chunkSize = 1024
  const stream = renderer.renderPdfStream(
    {
      template: '= Large Stream Document\n' + 'Typst streaming chunk verification.\n'.repeat(50),
    },
    chunkSize,
  )

  const chunks: Buffer[] = []
  await new Promise<void>((resolve, reject) => {
    stream.on('data', (chunk: Buffer) => {
      t.true(Buffer.isBuffer(chunk))
      t.true(chunk.length <= chunkSize)
      chunks.push(chunk)
    })
    stream.on('end', () => resolve())
    stream.on('error', (err) => reject(err))
  })

  t.true(chunks.length > 1, 'Expected multiple chunks from stream')
  const completePdf = Buffer.concat(chunks)
  t.is(completePdf.subarray(0, 5).toString('ascii'), '%PDF-')
})

test('TypstRenderer propagates compilation error to renderPdfStream error event', async (t) => {
  const renderer = new TypstRenderer()
  const stream = renderer.renderPdfStream({
    template: '= Error Stream\n#invalid_function_call_error_trigger()',
  })

  await t.throwsAsync(
    async () => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      for await (const _ of stream) {
        // Consume stream to trigger error
      }
    },
    { message: /invalid_function_call_error_trigger/ },
  )
})

test('TypstRenderer streams PNG pages via renderPngStream', async (t) => {
  const renderer = new TypstRenderer()
  const stream = renderer.renderPngStream({
    template: '= Page One\n#pagebreak()\n= Page Two',
    ppi: 72,
  })

  const pages: Array<{ page: number; data: Buffer }> = []
  for await (const item of stream) {
    pages.push(item as { page: number; data: Buffer })
  }

  t.is(pages.length, 2)
  t.is(pages[0].page, 1)
  t.is(pages[1].page, 2)
  // Verify PNG magic bytes on each streamed page
  t.deepEqual(Array.from(pages[0].data.subarray(0, 8)), [137, 80, 78, 71, 13, 10, 26, 10])
  t.deepEqual(Array.from(pages[1].data.subarray(0, 8)), [137, 80, 78, 71, 13, 10, 26, 10])
})

test('TypstRenderer streams SVG pages via renderSvgStream', async (t) => {
  const renderer = new TypstRenderer()
  const stream = renderer.renderSvgStream({
    template: '= Page Alpha\n#pagebreak()\n= Page Beta',
  })

  const pages: Array<{ page: number; svg: string }> = []
  for await (const item of stream) {
    pages.push(item as { page: number; svg: string })
  }

  t.is(pages.length, 2)
  t.is(pages[0].page, 1)
  t.true(pages[0].svg.includes('<svg'))
  t.is(pages[1].page, 2)
  t.true(pages[1].svg.includes('<svg'))
})
