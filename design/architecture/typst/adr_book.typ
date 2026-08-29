// Architecture Decision Records — print edition.
//
// Data comes from `scripts/adr_book.py`, which reads every record through
// syon-cli and structures the prose. Nothing here parses; it only lays out.

#let data = json("/build/adr/adrs.json")
#let records = data.records

#let ink = rgb("#1b1b1f")
#let quiet = rgb("#6b6b76")
#let rule = rgb("#d8d8de")
#let accent = rgb("#2b4a7d")

#let status-colour(s) = {
  if s == "accepted" { rgb("#1a7f37") } else if s == "proposed" { rgb("#9a6700") } else { quiet }
}

#let badge(s) = {
  let c = status-colour(s)
  box(
    fill: c.lighten(88%),
    stroke: 0.5pt + c.lighten(40%),
    inset: (x: 5pt, y: 2.5pt),
    radius: 2pt,
    text(size: 7.5pt, fill: c.darken(10%), weight: 600, tracking: 0.4pt, upper(s)),
  )
}

// --- inline runs ------------------------------------------------------------

#let render-runs(rs) = {
  for r in rs {
    if r.t == "code" {
      // outset rather than inset, so the tinted box does not widen the run
      // and a code span sitting at the start of a line stays flush.
      box(
        fill: rgb("#f2f2f5"),
        outset: (x: 1.5pt, y: 2pt),
        radius: 1.5pt,
        raw(r.v),
      )
    } else [#r.v]
  }
}

#let render-blocks(bs) = {
  for b in bs {
    if b.kind == "code" {
      block(
        width: 100%,
        fill: rgb("#f7f7f9"),
        stroke: (left: 2pt + rule),
        inset: (x: 10pt, y: 8pt),
        radius: 2pt,
        above: 10pt,
        below: 10pt,
        raw(b.text),
      )
    } else {
      par(render-runs(b.runs))
    }
  }
}

#let bullets(items, marker: none) = {
  for it in items {
    grid(
      columns: (11pt, 1fr),
      column-gutter: 0pt,
      align: (left + top, left + top),
      text(fill: marker.at(1), weight: 700)[#marker.at(0)],
      render-runs(it),
    )
    v(3.5pt, weak: true)
  }
}

#let section(title) = {
  v(9pt, weak: true)
  text(size: 8.5pt, weight: 600, fill: accent, tracking: 0.8pt, upper(title))
  v(1pt, weak: true)
  line(length: 100%, stroke: 0.5pt + rule)
  v(4pt, weak: true)
}

// --- page setup -------------------------------------------------------------

#set document(title: "Architecture Decision Records", author: "felix")

#set page(
  paper: "a4",
  margin: (top: 22mm, bottom: 20mm, x: 24mm),
  header: context {
    let p = here().page()
    if p <= 2 { return }
    let hs = query(heading.where(level: 1))
    let cur = none
    for h in hs { if h.location().page() <= p { cur = h } }
    if cur == none { return }
    set text(size: 8pt, fill: quiet)
    grid(
      columns: (1fr, auto),
      align: (left, right),
      emph(cur.body),
      [Architecture Decision Records],
    )
    v(2pt)
    line(length: 100%, stroke: 0.4pt + rule)
  },
  footer: context {
    let p = here().page()
    if p == 1 { return }
    set text(size: 8pt, fill: quiet)
    align(center)[#p]
  },
)

#set text(font: ("Libertinus Serif", "New Computer Modern", "Georgia", "Times New Roman"), size: 10pt, fill: ink, lang: "en")
#set par(justify: true, leading: 0.62em, spacing: 0.95em, first-line-indent: 0pt)
#show raw: set text(size: 8.7pt)  // Typst's bundled DejaVu Sans Mono, so no host font is assumed
#show link: set text(fill: accent)

// The level-1 heading exists for the outline, the PDF bookmarks and the
// running header. It is drawn as the eyebrow above each title rather than as
// a second title line.
#show heading.where(level: 1): it => block(above: 0pt, below: 5pt)[
  #text(size: 8.5pt, fill: quiet, tracking: 1.1pt, weight: 500)[#upper(it.body)]
]

// --- cover ------------------------------------------------------------------

#page(header: none, footer: none)[
  #v(1fr)
  #text(size: 30pt, weight: 600)[Architecture Decision Records]
  #v(4pt)
  #text(size: 13pt, fill: quiet)[SYON — Safe YAML Object Notation, and the layers above it]
  #v(14pt)
  #line(length: 38%, stroke: 1pt + accent)
  #v(14pt)
  #text(size: 10pt, fill: quiet)[
    #records.len() records · #records.filter(r => r.status == "accepted").len() accepted ·
    #records.filter(r => r.status == "proposed").len() proposed \
    Generated #datetime.today().display("[day] [month repr:long] [year]")
  ]
  #v(1fr)
  #text(size: 9pt, fill: quiet)[
    Every record on the following pages is a SYON document, read through
    `syon-cli` and validated in CI by both the Rust and the Go implementation.
  ]
]

// --- index ------------------------------------------------------------------

#[
  #text(size: 17pt, weight: 600)[Index]
  #v(8pt)
  #set text(size: 9pt)
  #table(
    columns: (auto, 1fr, auto),
    stroke: none,
    inset: (x: 4pt, y: 5pt),
    align: (left + top, left + top, right + top),
    fill: (_, y) => if calc.odd(y) { rgb("#fafafc") },
    table.header(
      text(weight: 600, size: 8pt, fill: quiet, tracking: 0.6pt)[ADR],
      text(weight: 600, size: 8pt, fill: quiet, tracking: 0.6pt)[TITLE],
      text(weight: 600, size: 8pt, fill: quiet, tracking: 0.6pt)[STATUS],
    ),
    ..records.map(r => (
      link(label(r.id))[#raw(r.id)],
      link(label(r.id))[#render-runs(r.title)],
      badge(r.status),
    )).flatten()
  )
]

// --- the records ------------------------------------------------------------

#for r in records {
  pagebreak()

  [
    #heading(level: 1, outlined: true, bookmarked: true)[#r.id]
    #label(r.id)
  ]

  block(above: 0pt, below: 8pt)[
    #text(size: 16pt, weight: 600)[#render-runs(r.title)]
  ]

  block(
    width: 100%,
    fill: rgb("#fafafc"),
    stroke: (left: 2pt + status-colour(r.status).lighten(50%)),
    inset: (x: 10pt, y: 7pt),
    radius: 2pt,
    below: 12pt,
  )[
    #set text(size: 8.5pt, fill: quiet)
    #grid(
      columns: (auto, 1fr),
      column-gutter: 8pt,
      row-gutter: 4pt,
      [Status], badge(r.status),
      [Date], [#r.date],
      [Deciders], [#r.deciders.join(", ")],
      ..if r.superseded_by != "" { ([Superseded by], raw(r.superseded_by)) } else { () },
      ..if r.amended_by != "" { ([Amended by], raw(r.amended_by)) } else { () },
    )
  ]

  if r.open_questions.len() > 0 {
    section("Open questions")
    bullets(r.open_questions, marker: ("?", rgb("#9a6700")))
  }

  section("Context")
  render-blocks(r.context)

  section("Decision")
  render-blocks(r.decision)

  if r.outcome.len() > 0 {
    section("Outcome")
    render-blocks(r.outcome)
  }

  section("Consequences")
  bullets(r.positive, marker: ("+", rgb("#1a7f37")))
  if r.positive.len() > 0 and r.negative.len() > 0 { v(3pt) }
  bullets(r.negative, marker: ("−", rgb("#b42318")))

  if r.rejected.len() > 0 {
    section("Alternatives rejected")
    for a in r.rejected {
      block(below: 7pt, breakable: false)[
        #text(weight: 600, size: 9.5pt)[#render-runs(a.name)] \
        #set text(size: 9.5pt, fill: ink.lighten(15%))
        #render-runs(a.reason)
      ]
    }
  }
}
