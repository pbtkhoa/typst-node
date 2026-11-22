import { simpleFixture, mediumFixture, complexFixture } from './fixtures.js'

export interface CompetitorAdapter {
  id: string
  name: string
  isAvailable: () => Promise<boolean>
  setup?: () => Promise<void>
  teardown?: () => Promise<void>
  renderSimple: () => Promise<Buffer>
  renderMedium: () => Promise<Buffer>
  renderComplex: () => Promise<Buffer>
  installInstruction: string
}

/**
 * Competitor Adapter for PDFKit (Pure JavaScript imperative PDF generation)
 */
export class PdfKitAdapter implements CompetitorAdapter {
  id = 'pdfkit'
  name = 'PDFKit (Pure JS)'
  installInstruction = 'yarn add -D pdfkit @types/pdfkit'

  private pdfkitModule: any = null

  async isAvailable(): Promise<boolean> {
    try {
      // @ts-ignore dynamic check
      const mod = await import('pdfkit')
      this.pdfkitModule = mod.default || mod
      return true
    } catch {
      return false
    }
  }

  private bufferFromDoc(doc: any): Promise<Buffer> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = []
      doc.on('data', (chunk: Buffer) => chunks.push(chunk))
      doc.on('end', () => resolve(Buffer.concat(chunks)))
      doc.on('error', reject)
    })
  }

  async renderSimple(): Promise<Buffer> {
    const PDFDocument = this.pdfkitModule
    const doc = new PDFDocument({ size: 'A6', margin: 20 })
    const promise = this.bufferFromDoc(doc)

    const data = simpleFixture.data
    doc.fontSize(11).text(data.storeName, { align: 'center' })
    doc.fontSize(7).text(`Store #${data.storeId} -- ${data.address}`, { align: 'center' })
    doc.moveDown(0.5)
    doc.fontSize(8).text(`Order: #${data.orderId}            ${data.date}`)
    doc.moveDown(0.5)

    doc.text('----------------------------------------------')
    for (const item of data.items) {
      doc.text(`${item.name} x${item.qty}   $${item.price.toFixed(2)}`)
    }
    doc.text('----------------------------------------------')
    doc.text(`Subtotal: $${data.subtotal}`)
    doc.text(`Tax: $${data.tax}`)
    doc.fontSize(9).text(`Total: $${data.total}`)
    doc.end()

    return promise
  }

  async renderMedium(): Promise<Buffer> {
    const PDFDocument = this.pdfkitModule
    const doc = new PDFDocument({ size: 'A4', margin: 50 })
    const promise = this.bufferFromDoc(doc)

    const data = mediumFixture.data
    // Page 1
    doc.fontSize(18).text(`INVOICE #${data.invoiceNumber}`)
    doc.fontSize(10).text(`Date: ${data.issueDate} | Due: ${data.dueDate}`)
    doc.text(`Bill To: ${data.recipient.name}`)
    doc.moveDown()
    for (const item of data.lineItems) {
      doc.text(`${item.id}. ${item.description} (x${item.quantity}) - $${item.amount}`)
    }

    // Page 2
    doc.addPage()
    doc.fontSize(14).text('Milestone Breakdown')
    for (const m of data.milestones) {
      doc.text(`${m.phase}: ${m.objective} [${m.status}]`)
    }
    doc.moveDown()
    doc.text('Engineer Logs:')
    for (const h of data.hours) {
      doc.text(`${h.resource} (${h.role}): ${h.hours} hrs @ $${h.rate}/hr`)
    }

    // Page 3
    doc.addPage()
    doc.fontSize(14).text('Settlement & Wire Instructions')
    doc.fontSize(10).text(`Total Amount Due: $${data.totalDue}`)
    doc.text('Silicon Valley Commercial Bank - Account #8492049102')
    doc.end()

    return promise
  }

  async renderComplex(): Promise<Buffer> {
    const PDFDocument = this.pdfkitModule
    const doc = new PDFDocument({ size: 'A4', margin: 50 })
    const promise = this.bufferFromDoc(doc)

    const data = complexFixture.data
    // Cover page
    doc.fontSize(24).text(data.reportTitle, { align: 'center' })
    doc.fontSize(14).text(data.subTitle, { align: 'center' })
    doc.moveDown()
    doc.fontSize(10).text(`Prepared for: ${data.targetAudience} | Date: ${data.generationDate}`, { align: 'center' })

    // 19 pages
    for (const sec of data.sections) {
      doc.addPage()
      doc.fontSize(16).text(`${sec.id}. ${sec.title}`)
      doc.fontSize(10).text(sec.overview)
      doc.moveDown()
      doc.text(`Throughput: ${sec.kpi1} | P99: ${sec.kpi2} | Error Rate: ${sec.kpi3}`)
      doc.moveDown()
      for (const m of sec.metrics) {
        doc.text(`${m.module}: P50 ${m.p50}ms | P95 ${m.p95}ms | P99 ${m.p99}ms [${m.availability}]`)
      }
    }
    doc.end()

    return promise
  }
}

/**
 * Competitor Adapter for Puppeteer (Headless Chromium)
 */
export class PuppeteerAdapter implements CompetitorAdapter {
  id = 'puppeteer'
  name = 'Puppeteer (Headless Chromium)'
  installInstruction = 'yarn add -D puppeteer'

  private puppeteerModule: any = null
  private browser: any = null
  private page: any = null

  async isAvailable(): Promise<boolean> {
    try {
      // @ts-ignore dynamic check
      const mod = await import('puppeteer')
      this.puppeteerModule = mod.default || mod

      const fs = await import('node:fs')
      const macChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
      const executablePath = fs.existsSync(macChrome) ? macChrome : undefined

      // Verify browser actually launches and connects without socket errors
      const browser = await this.puppeteerModule.launch({
        executablePath,
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage'],
      })
      await browser.close()
      return true
    } catch {
      return false
    }
  }

  async setup(): Promise<void> {
    if (!this.puppeteerModule) return
    const fs = await import('node:fs')
    const macChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
    const executablePath = fs.existsSync(macChrome) ? macChrome : undefined

    this.browser = await this.puppeteerModule.launch({
      executablePath,
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage'],
    })
    this.page = await this.browser.newPage()
  }

  async teardown(): Promise<void> {
    if (this.page) await this.page.close().catch(() => {})
    if (this.browser) await this.browser.close().catch(() => {})
  }

  async renderSimple(): Promise<Buffer> {
    const data = simpleFixture.data
    const html = `
      <!DOCTYPE html>
      <html>
      <head>
        <style>
          @page { size: A6; margin: 8mm; }
          body { font-family: monospace; font-size: 10px; margin: 0; }
          .center { text-align: center; }
          .right { text-align: right; }
          table { width: 100%; border-collapse: collapse; }
          hr { border: none; border-top: 1px dashed #000; margin: 5px 0; }
        </style>
      </head>
      <body>
        <div class="center">
          <h3>${data.storeName}</h3>
          <p>Store #${data.storeId} - ${data.address}</p>
        </div>
        <hr/>
        <div>Order: #${data.orderId} <span style="float:right">${data.date}</span></div>
        <hr/>
        <table>
          ${data.items.map((it: any) => `<tr><td>${it.name} x${it.qty}</td><td class="right">$${it.price.toFixed(2)}</td></tr>`).join('')}
        </table>
        <hr/>
        <div class="right">
          <div>Subtotal: $${data.subtotal}</div>
          <div>Tax: $${data.tax}</div>
          <b>Total: $${data.total}</b>
        </div>
      </body>
      </html>
    `
    await this.page.setContent(html, { waitUntil: 'load' })
    const pdf = await this.page.pdf({ format: 'A6' })
    return Buffer.from(pdf)
  }

  async renderMedium(): Promise<Buffer> {
    const data = mediumFixture.data
    const html = `
      <!DOCTYPE html>
      <html>
      <head>
        <style>
          @page { size: A4; margin: 20mm; }
          body { font-family: serif; font-size: 12px; }
          .pagebreak { page-break-before: always; }
          table { width: 100%; border-collapse: collapse; margin-top: 10px; }
          th, td { border-bottom: 1px solid #ccc; padding: 6px; text-align: left; }
        </style>
      </head>
      <body>
        <h1>INVOICE #${data.invoiceNumber}</h1>
        <p>Date: ${data.issueDate} | Due: ${data.dueDate}</p>
        <table>
          <tr><th>Item</th><th>Qty</th><th>Price</th></tr>
          ${data.lineItems.map((it: any) => `<tr><td>${it.description}</td><td>${it.quantity}</td><td>$${it.amount}</td></tr>`).join('')}
        </table>
        <div class="pagebreak"></div>
        <h2>Milestone Logs</h2>
        <table>
          ${data.milestones.map((m: any) => `<tr><td>${m.phase}</td><td>${m.objective}</td><td>${m.status}</td></tr>`).join('')}
        </table>
        <div class="pagebreak"></div>
        <h2>Settlement Instructions</h2>
        <p>Total Due: $${data.totalDue}</p>
      </body>
      </html>
    `
    await this.page.setContent(html, { waitUntil: 'load' })
    const pdf = await this.page.pdf({ format: 'A4' })
    return Buffer.from(pdf)
  }

  async renderComplex(): Promise<Buffer> {
    const data = complexFixture.data
    const html = `
      <!DOCTYPE html>
      <html>
      <head>
        <style>
          @page { size: A4; margin: 20mm; }
          body { font-family: serif; font-size: 11px; }
          .pagebreak { page-break-before: always; }
          table { width: 100%; border-collapse: collapse; margin-top: 10px; }
          th, td { border-bottom: 1px solid #ddd; padding: 4px; text-align: left; }
        </style>
      </head>
      <body>
        <div style="height: 80vh; display: flex; flex-direction: column; justify-content: center; align-items: center;">
          <h1>${data.reportTitle}</h1>
          <h3>${data.subTitle}</h3>
        </div>
        ${data.sections
          .map(
            (sec: any) => `
          <div class="pagebreak"></div>
          <h2>${sec.id}. ${sec.title}</h2>
          <p>${sec.overview}</p>
          <table>
            <tr><th>Module</th><th>P50</th><th>P95</th><th>P99</th></tr>
            ${sec.metrics.map((m: any) => `<tr><td>${m.module}</td><td>${m.p50}ms</td><td>${m.p95}ms</td><td>${m.p99}ms</td></tr>`).join('')}
          </table>
        `,
          )
          .join('')}
      </body>
      </html>
    `
    await this.page.setContent(html, { waitUntil: 'load' })
    const pdf = await this.page.pdf({ format: 'A4' })
    return Buffer.from(pdf)
  }
}
