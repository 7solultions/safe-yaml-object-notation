# 4. Independent Go implementation instead of FFI bindings

Date: 2026-07-11

## Status

Accepted

## Context

SYON has three language surfaces:

- **Rust** (`crates/syon-parser`, `crates/syon-cli`) — the primary
  implementation.
- **Python** (`crates/syon-python`) — a PyO3 extension module that binds
  directly to `syon-parser`. It has no independent parsing logic; it
  converts the Rust `Value` tree to Python objects.
- **Go** (`syon-go`) — a from-scratch, dependency-free, cgo-free parser
  and encoder. It does not call into the Rust crate at all; it has its
  own hand-written recursive-descent parser
  (`syon-go/syon.go`) implementing the spec independently.

Given `syon-parser` already exposes a C ABI (`crates/syon-parser/src/ffi.rs`,
`cdylib`/`staticlib` outputs) for exactly this kind of cross-language
reuse, Go bindings via cgo were a real alternative to writing a second
parser from scratch.

## Decision

Give Go an independent, pure-Go implementation rather than cgo bindings
to `syon-parser`.

## Consequences

- Go consumers get a dependency-free, cgo-free module — no C toolchain,
  no cross-compilation pain from linking a Rust cdylib, no CGO_ENABLED
  gotchas. This matters more for a Go audience than for Python, where
  PyO3/maturin-built wheels already hide the FFI boundary from consumers.
- SYON's grammar and forbidden-construct rules are now implemented
  *twice*, independently, in Rust and in Go. They can drift apart. There
  is no shared conformance test suite run against both implementations —
  `crates/syon-parser`'s tests and `syon-go`'s tests are separate, and
  only agree by construction on the two example fixtures each parses in
  `examples_test.go` / the CI `examples-valid` and `go-build` jobs (see
  `.github/workflows/ci.yml`).
- Any spec change (new forbidden construct, new block type, semantics
  fix) needs to be ported to both implementations by hand; nothing
  enforces that they stay in sync beyond spec prose and code review.
- If this drift becomes a real problem, a shared conformance test corpus
  (input SYON documents + expected AST/error, run against every
  implementation) would be the natural follow-up — not currently done.
