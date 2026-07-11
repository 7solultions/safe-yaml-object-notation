# Architecture Decision Records

This directory records the architecturally significant decisions made on
this project, in the lightweight [Michael Nygard ADR
format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions).

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-pest-as-the-rust-parsing-engine.md) | Use pest as the Rust parsing engine | Accepted |
| [0003](0003-preflight-scan-for-forbidden-constructs.md) | Preflight text scan for forbidden constructs | Accepted |
| [0004](0004-independent-go-implementation.md) | Independent Go implementation instead of FFI bindings | Accepted |
| [0005](0005-block-1-only-yaml-compatibility.md) | Only Block 1 is YAML-compatible | Accepted |

## Adding a new ADR

Copy the format of an existing record: `NNNN-short-title.md`, numbered
sequentially. Never edit or delete an accepted ADR to reflect a later
change of mind — write a new one that supersedes it, and mark the old one's
status accordingly (`Superseded by ADR-00NN`).
