# 1. Record architecture decisions

Date: 2026-07-11

## Status

Accepted

## Context

SYON has accumulated several architecturally significant decisions —
which parsing engine to use, why forbidden constructs are rejected via a
separate text scan rather than the grammar alone, why the Go
implementation doesn't just bind to the Rust library, how far YAML
compatibility actually goes — that were never written down anywhere. They
existed only as commit messages and code comments, which meant new
contributors (human or otherwise) had to reconstruct the reasoning from
scratch, or risked re-litigating settled trade-offs.

## Decision

Record architecturally significant decisions as Architecture Decision
Records (ADRs), using Michael Nygard's lightweight format, in
`docs/decisions/`.

Each ADR is a short Markdown file: `NNNN-short-title.md`, numbered
sequentially, with Status / Context / Decision / Consequences sections.
Once accepted, an ADR is not edited to reflect a later reversal — a new
ADR supersedes it instead, so the log reads as a true history rather than
the current-best-guess overwritten in place.

## Consequences

- Decisions and their rationale are discoverable in one place instead of
  scattered across git history and spec prose.
- Every new architecturally significant decision should get an ADR going
  forward; this is a discipline, not a one-time backfill.
- The log will contain decisions that were later superseded — that's
  intentional, not a maintenance burden to "clean up".
