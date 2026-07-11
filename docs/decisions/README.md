# Architecture Decision Records

This directory records the architecturally significant decisions made on
this project, in a lightweight [Michael Nygard ADR
format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
adapted to fields, expressed as SYON documents — each ADR is itself a piece
of real SYON content, validated by both the Rust and Go implementations in
CI (see `examples-valid` and `go-build` in
[`.github/workflows/ci.yml`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/.github/workflows/ci.yml)).

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/docs/decisions/0001-record-architecture-decisions.syon) | Record architecture decisions | Accepted |
| [0002](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/docs/decisions/0002-pest-as-the-rust-parsing-engine.syon) | Use pest as the Rust parsing engine | Accepted |
| [0003](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/docs/decisions/0003-preflight-scan-for-forbidden-constructs.syon) | Preflight text scan for forbidden constructs | Accepted |
| [0004](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/docs/decisions/0004-independent-go-implementation.syon) | Independent Go implementation instead of FFI bindings | Accepted |
| [0005](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/docs/decisions/0005-block-1-only-yaml-compatibility.syon) | Only Block 1 is YAML-compatible | Accepted |

## Record schema

Each ADR is a `.syon` file with one top-level `architecture-decision-record`
mapping:

```syon
architecture-decision-record:
  identifier: "0006"
  title: Example decision title
  status: accepted
  date: "2026-07-11"
  deciders:
    - felix
  superseded-by: ""
  context: [[[
    What's the issue we're seeing that motivates this decision?
  ]]]
  decision: [[[
    What are we going to do about it?
  ]]]
  consequences:
    positive:
      - A good outcome of this decision.
    negative:
      - A trade-off or cost accepted along with it.
  alternatives-rejected:
    -
      name: An option that was considered
      reason: Why it wasn't chosen.
```

Note the `-` starting an `alternatives-rejected` entry must be alone on its
own line, with the nested `name`/`reason` mapping indented underneath — SYON
does not currently support a mapping key starting on the same line as the
list marker (e.g. `- name: ...`).

## Adding a new ADR

Copy this schema into a new `NNNN-short-title.syon` file, numbered
sequentially, and add a row to the index above. Never edit or delete an
accepted ADR to reflect a later change of mind — write a new one that
supersedes it (setting the new ADR's context to reference the old one, and
the old ADR's `superseded-by` field to the new one's identifier).
