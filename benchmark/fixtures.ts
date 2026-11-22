export interface BenchmarkFixture {
  name: string
  description: string
  template: string
  data: Record<string, any>
  assets?: Record<string, Buffer>
}

// 1x1 transparent PNG buffer for fixtures that require image assets
const DUMMY_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
  'base64',
)

export const simpleFixture: BenchmarkFixture = {
  name: 'Simple (1-page Receipt)',
  description: 'Single-page POS receipt with dynamic items, taxes, and totals',
  template: `
    #set page(paper: "a6", margin: (x: 8mm, y: 10mm))
    #set text(font: "DejaVu Sans Mono", size: 8pt)

    #let data = sys.inputs

    #align(center)[
      #text(size: 11pt, weight: "bold")[#data.storeName] \\
      #text(size: 7pt)[Store \\# #data.storeId -- #data.address] \\
      #line(length: 100%, stroke: 0.5pt)
    ]

    #grid(
      columns: (1fr, 1fr),
      [Order: \\##data.orderId],
      align(right)[#data.date],
    )

    #v(2mm)
    #table(
      columns: (3fr, 1fr, 1.5fr),
      stroke: none,
      table.hline(stroke: 0.5pt),
      [*Item*], [*Qty*], align(right)[*Price*],
      table.hline(stroke: 0.5pt),
      ..data.items.map(it => (
        it.name,
        str(it.qty),
        align(right)[\\$#calc.round(it.price, digits: 2)]
      )).flatten(),
      table.hline(stroke: 0.5pt),
    )

    #v(2mm)
    #align(right)[
      #block(width: 60%)[
        #grid(
          columns: (1fr, 1fr),
          [Subtotal:], align(right)[\\$#data.subtotal],
          [Tax (8.25%):], align(right)[\\$#data.tax],
          grid.hline(stroke: 0.5pt),
          [*Total:*], align(right)[*\\$#data.total*],
        )
      ]
    ]

    #v(4mm)
    #align(center)[
      #text(size: 7pt)[Thank you for your business!] \\
      #text(size: 6pt)[Card ending in #data.cardLast4]
    ]
  `,
  data: {
    storeName: 'Acme Coffee & Roastery',
    storeId: '402',
    address: '101 Market St, San Francisco, CA',
    orderId: 'ORD-98214',
    date: '2026-11-22 14:32',
    cardLast4: '4242',
    items: [
      { name: 'Single Origin Espresso', qty: 2, price: 4.5 },
      { name: 'Oat Milk Flat White', qty: 1, price: 5.75 },
      { name: 'Almond Croissant', qty: 2, price: 4.25 },
      { name: 'Dark Roast Whole Beans 250g', qty: 1, price: 18.0 },
    ],
    subtotal: 41.25,
    tax: 3.4,
    total: 44.65,
  },
}

export const mediumFixture: BenchmarkFixture = {
  name: 'Medium (3-page B2B Invoice)',
  description: '3-page commercial invoice with dynamic tables, multi-page layout, and image asset',
  template: `
    #set page(
      paper: "a4",
      margin: (x: 2cm, y: 2.5cm),
      header: align(right)[
        #text(size: 8pt, fill: luma(120))[Invoice \\##sys.inputs.invoiceNumber -- Page #context counter(page).display()]
      ],
      footer: align(center)[
        #text(size: 8pt, fill: luma(150))[Payment due within 30 days of issuance. Wire transfer details on page 3.]
      ]
    )
    #set text(font: "Libertinus Serif", size: 10pt)

    #let data = sys.inputs

    #grid(
      columns: (1fr, auto),
      [
        #text(size: 20pt, weight: "bold", fill: rgb("1d3557"))[INVOICE] \\
        #text(size: 11pt, weight: "medium")[Invoice \\##data.invoiceNumber] \\
        #v(1mm)
        #text(size: 9pt)[
          *Date:* #data.issueDate \\
          *Due Date:* #data.dueDate \\
          *PO Number:* #data.poNumber
        ]
      ],
      [
        #image("logo.png", width: 35mm) \\
        #text(size: 8pt)[
          *#data.issuer.name* \\
          #data.issuer.address \\
          #data.issuer.taxId
        ]
      ]
    )

    #v(5mm)
    #line(length: 100%, stroke: 1pt + rgb("1d3557"))
    #v(2mm)

    #grid(
      columns: (1fr, 1fr),
      [
        #text(weight: "bold", size: 10pt)[Bill To:] \\
        #data.recipient.name \\
        #data.recipient.department \\
        #data.recipient.address \\
        #data.recipient.email
      ],
      [
        #text(weight: "bold", size: 10pt)[Payment Terms:] \\
        Net 30 Days \\
        Currency: #data.currency \\
        Billing Contact: #data.issuer.contact
      ]
    )

    #v(8mm)
    #text(weight: "bold", size: 12pt)[Line Items & Deliverables]
    #v(2mm)

    #table(
      columns: (1fr, 4fr, 1fr, 1.5fr, 1.5fr),
      fill: (col, row) => if row == 0 { rgb("eef2f6") } else if calc.even(row) { rgb("fafafa") } else { none },
      stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("1d3557")) } else { (bottom: 0.5pt + luma(220)) },
      [*Item*], [*Description*], [*Qty*], [*Unit Price*], align(right)[*Amount*],
      ..data.lineItems.map(it => (
        str(it.id),
        it.description,
        str(it.quantity),
        [\\$#str(it.unitPrice)],
        align(right)[\\$#str(it.amount)]
      )).flatten()
    )

    #pagebreak()

    #heading(level: 2)[Milestone Breakdown & Service Logs]
    #v(3mm)
    #table(
      columns: (2fr, 4fr, 2fr),
      fill: (col, row) => if row == 0 { rgb("eef2f6") } else { none },
      [*Phase*], [*Objective*], [*Status*],
      ..data.milestones.map(m => (
        m.phase,
        m.objective,
        m.status
      )).flatten()
    )

    #v(6mm)
    #heading(level: 2)[Detailed Hours by Engineer]
    #v(2mm)
    #table(
      columns: (3fr, 2fr, 2fr, 2fr),
      fill: (col, row) => if row == 0 { rgb("eef2f6") } else { none },
      [*Resource*], [*Role*], [*Hours*], [*Rate*],
      ..data.hours.map(h => (
        h.resource,
        h.role,
        str(h.hours),
        [\\$#str(h.rate)/hr]
      )).flatten()
    )

    #pagebreak()

    #heading(level: 2)[Settlement & Wire Instructions]
    #v(3mm)
    #grid(
      columns: (1.5fr, 1fr),
      [
        #text(weight: "bold")[Bank Transfer Instructions:] \\
        Bank Name: Silicon Valley Commercial Bank \\
        Account Name: Acme Enterprise Services Inc. \\
        Account Number: 8492049102 \\
        Routing Number: 121000358 \\
        SWIFT / BIC: SVCBUS33 \\
        Reference: INV-2026-8812
      ],
      [
        #rect(fill: rgb("f1faee"), inset: 4mm, radius: 2mm)[
          #align(center)[
            #text(size: 11pt, weight: "bold")[Total Amount Due] \\
            #v(1mm)
            #text(size: 16pt, weight: "bold", fill: rgb("1d3557"))[\\$#data.totalDue] \\
            #v(1mm)
            #text(size: 8pt)[Due by: #data.dueDate]
          ]
        ]
      ]
    )

    #v(8mm)
    #heading(level: 3)[Terms & Compliance]
    #text(size: 8pt, fill: luma(100))[
      All services provided are subject to our Master Services Agreement dated January 15, 2026. Late payments are subject to a 1.5% monthly compounding interest fee. Disputed items must be submitted in writing within 10 business days of invoice receipt.
    ]
  `,
  data: {
    invoiceNumber: 'INV-2026-8812',
    issueDate: '2026-11-22',
    dueDate: '2026-12-22',
    poNumber: 'PO-99120',
    currency: 'USD',
    totalDue: '64,500.00',
    issuer: {
      name: 'Acme Cloud Engineering Inc.',
      address: '500 Howard Street, Suite 400, San Francisco, CA 94105',
      taxId: 'US-EIN-94-3829104',
      contact: 'billing@acmecloud.example.com',
    },
    recipient: {
      name: 'Global Fintech Enterprises LLC',
      department: 'Infrastructure Platform Group',
      address: '1200 Avenue of the Americas, 18th Floor, New York, NY 10036',
      email: 'accounts-payable@globalfintech.example.com',
    },
    lineItems: [
      {
        id: 1,
        description: 'Core Typst Compilation Engine Integration (Phase 1)',
        quantity: 1,
        unitPrice: 22000,
        amount: 22000,
      },
      {
        id: 2,
        description: 'N-API Async Worker Pool Implementation & Benchmarking',
        quantity: 1,
        unitPrice: 18500,
        amount: 18500,
      },
      {
        id: 3,
        description: 'Zero Disk I/O In-Memory Virtual Asset Loader & Caching',
        quantity: 1,
        unitPrice: 12000,
        amount: 12000,
      },
      {
        id: 4,
        description: 'End-to-End Multi-Tenant Performance Testing & Profiling',
        quantity: 1,
        unitPrice: 8000,
        amount: 8000,
      },
      {
        id: 5,
        description: 'Architectural Documentation & Handover Workshop',
        quantity: 1,
        unitPrice: 4000,
        amount: 4000,
      },
    ],
    milestones: [
      {
        phase: 'Phase 1 - Engine Core',
        objective: 'Pure Rust typst::World and MemoryWorld implementation',
        status: 'Completed',
      },
      {
        phase: 'Phase 2 - Node Bridge',
        objective: 'napi::Task async worker threads for non-blocking I/O',
        status: 'Completed',
      },
      {
        phase: 'Phase 3 - Asset System',
        objective: 'Rootless virtual path resolution for #include & #image',
        status: 'Completed',
      },
      {
        phase: 'Phase 4 - Verification',
        objective: 'Comprehensive test suites and latency benchmarks',
        status: 'In Progress',
      },
    ],
    hours: [
      { resource: 'Sarah Jenkins', role: 'Principal Systems Architect', hours: 80, rate: 250 },
      { resource: 'Marcus Vance', role: 'Senior Rust Engineer', hours: 120, rate: 200 },
      { resource: 'Elena Rostova', role: 'Node.js N-API Specialist', hours: 75, rate: 180 },
      { resource: 'David Kim', role: 'Performance & QA Engineer', hours: 50, rate: 140 },
    ],
  },
  assets: {
    'logo.png': DUMMY_PNG,
  },
}

export const complexFixture: BenchmarkFixture = {
  name: 'Complex (20-page Analytics Report)',
  description: '20-page technical report with 20 sections, dynamic tables, headers, and metrics',
  template: `
    #set page(
      paper: "a4",
      margin: (x: 2cm, y: 2.2cm),
      header: context {
        let pageNum = counter(page).get().first()
        if pageNum > 1 [
          #grid(
            columns: (1fr, 1fr),
            text(size: 8pt, fill: luma(120))[#sys.inputs.reportTitle],
            align(right)[#text(size: 8pt, fill: luma(120))[Page #pageNum of 20]]
          )
          #line(length: 100%, stroke: 0.5pt + luma(200))
        ]
      },
      footer: align(center)[
        #text(size: 8pt, fill: luma(160))[Confidential -- Internal Distribution Only]
      ]
    )
    #set text(font: "Libertinus Serif", size: 10pt)

    #let data = sys.inputs

    // Cover Page
    #align(center + horizon)[
      #text(size: 26pt, weight: "bold", fill: rgb("003049"))[#data.reportTitle] \\
      #v(4mm)
      #text(size: 14pt, fill: rgb("669bbc"))[#data.subTitle] \\
      #v(8mm)
      #line(length: 40%, stroke: 2pt + rgb("c1121f")) \\
      #v(8mm)
      #text(size: 11pt)[*Prepared For:* #data.targetAudience] \\
      #text(size: 10pt)[*Date:* #data.generationDate] \\
      #text(size: 10pt)[*Version:* #data.version]
    ]

    // 19 Subsequent Pages (Total 20 Pages)
    #for sec in data.sections [
      #pagebreak()
      #heading(level: 1)[#sec.id. #sec.title]
      #v(2mm)
      #text(size: 10pt)[#sec.overview]
      #v(4mm)

      #grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 3mm,
        rect(fill: rgb("edf2f4"), inset: 3mm, radius: 1mm)[
          #align(center)[
            #text(size: 8pt)[Throughput] \\
            #text(size: 14pt, weight: "bold", fill: rgb("003049"))[#sec.kpi1]
          ]
        ],
        rect(fill: rgb("edf2f4"), inset: 3mm, radius: 1mm)[
          #align(center)[
            #text(size: 8pt)[P99 Latency] \\
            #text(size: 14pt, weight: "bold", fill: rgb("780000"))[#sec.kpi2]
          ]
        ],
        rect(fill: rgb("edf2f4"), inset: 3mm, radius: 1mm)[
          #align(center)[
            #text(size: 8pt)[Error Rate] \\
            #text(size: 14pt, weight: "bold", fill: rgb("2b9348"))[#sec.kpi3]
          ]
        ]
      )

      #v(5mm)
      #heading(level: 2)[Component Performance Metrics]
      #v(2mm)
      #table(
        columns: (3fr, 2fr, 2fr, 2fr, 2fr),
        fill: (col, row) => if row == 0 { rgb("e0e1dd") } else if calc.even(row) { rgb("f8f9fa") } else { none },
        [*Module*], [*P50 (ms)*], [*P95 (ms)*], [*P99 (ms)*], [*Availability*],
        ..sec.metrics.map(m => (
          m.module,
          str(m.p50),
          str(m.p95),
          str(m.p99),
          m.availability
        )).flatten()
      )

      #v(4mm)
      #text(size: 9pt, style: "italic", fill: luma(100))[
        Observations: #sec.observations
      ]
    ]
  `,
  data: {
    reportTitle: 'Global Microservices Performance Audit',
    subTitle: 'Q4 2026 Production Benchmark & Latency Analysis',
    targetAudience: 'Engineering Leadership & Infrastructure Guild',
    generationDate: 'November 22, 2026',
    version: 'v4.18-final',
    sections: Array.from({ length: 19 }, (_, i) => ({
      id: i + 1,
      title: `Subsystem Cluster ${String.fromCharCode(65 + (i % 26))}${i + 1} Analysis`,
      overview: `Detailed throughput, p99 tail latency, and saturation telemetry collected from edge compute nodes across regional zones during the benchmark evaluation cycle.`,
      kpi1: `${(1200 + i * 85).toLocaleString()} req/s`,
      kpi2: `${(12.4 + ((i * 0.7) % 6)).toFixed(1)} ms`,
      kpi3: `${(0.01 + ((i * 0.005) % 0.05)).toFixed(3)}%`,
      metrics: [
        { module: 'Authentication Gateway', p50: 3.2, p95: 7.8, p99: 11.2, availability: '99.99%' },
        { module: 'Typst Compilation Worker', p50: 6.5, p95: 14.1, p99: 18.9, availability: '100.0%' },
        { module: 'Memory Font Cache', p50: 0.1, p95: 0.3, p99: 0.8, availability: '100.0%' },
        { module: 'Vector Asset Resolver', p50: 0.8, p95: 1.9, p99: 3.4, availability: '99.98%' },
        { module: 'Stream Serialization Pipeline', p50: 2.1, p95: 4.5, p99: 6.7, availability: '99.99%' },
      ],
      observations: `All service level objectives (SLOs) were satisfied under standard load conditions. Memory utilization remained bounded with zero disk spill.`,
    })),
  },
}
