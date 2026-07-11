# 5. Only Block 1 is YAML-compatible

Date: 2026-07-11

## Status

Accepted

## Context

Early project documentation (`spec/README.md`, the root `README.md`, and
the docs site) claimed "SYON is a strict safe subset of YAML 1.2" without
qualification. That claim doesn't hold for the whole language:

- **Block 1** (record syntax: indentation-based mappings/sequences using
  `: `, `- `, `# `) genuinely is a safe subset of YAML 1.2 block style —
  every valid Block-1-only document is valid YAML.
- **Block 2** (document fences, ` ```path.format ` … ` ``` `) and
  **Block 3** (literal escape hatch, `[[[` … `]]]`) are SYON-specific
  syntax with no YAML equivalent. A ` ```path.format ` fence or a
  `[[[`/`]]]` delimiter is not valid YAML syntax at all — a document using
  either block type cannot be parsed by a YAML 1.2 parser, full stop.

This mismatch wasn't caught until a project review of the generated docs
site pointed out that Block 2 provides multi-document-style embedding
(SYON deliberately has no YAML-native `---`/`...` markers, but Block 2
fences fill that role through different, non-YAML syntax) and that Block 3
is not YAML's `|` block scalar, despite serving a similar purpose.

## Decision

State the YAML relationship precisely: only Block 1 is a strict, safe
YAML 1.2 subset. SYON as a whole is not YAML-compatible, because of Block
2 and Block 3.

`spec/README.md`, `spec/02-grammar.md`'s forbidden-set table, the root
`README.md`, and `docs/index.md`/`docs/language.md` were corrected to
this effect (see the "Correct SYON-vs-YAML relationship claim" PR).

## Consequences

- Anyone reaching for a plain YAML 1.2 parser/library to consume
  arbitrary SYON documents will work fine for Block-1-only content, and
  will fail (or silently misinterpret) any document using a fence or a
  literal block. This must not be undersold in documentation again.
- Tracking issues [#2](https://github.com/object-notation-environment/safe-yaml-object-notation/issues/2)
  and [#3](https://github.com/object-notation-environment/safe-yaml-object-notation/issues/3)
  propose optional YAML-compatible modes for Block 2 and Block 3
  respectively. If either ships, this ADR should be superseded (or a new
  one added) rather than silently reworded, per ADR 0001's discipline.
- Every future doc page that describes SYON's relationship to YAML should
  link to `spec/README.md#relationship-to-yaml` rather than restating the
  claim inline, to avoid the same drift happening again in a second place.
