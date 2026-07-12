# Error codes

Every `SyonError` (Rust), `syon.Error` (Go), and `syon.SyonError` (Python)
carries a three-digit numeric code alongside its human-readable message, in
addition to the `Forbidden`/`Syntax` distinction from
[03-semantics.md](03-semantics.md)'s error model. The code is stable across
releases and is intended for programmatic dispatch (e.g. "is this a
duplicate-key error?") without string-matching the message.

## Terminology note: numbering follows the phase1 tool, not this spec

The ranges below number Block 2 as **literal blocks** (`[[[...]]]`) and
Block 3 as **document fences** (` ```path.format ` ). This is the
*opposite* of this spec's own numbering in
[02-grammar.md](02-grammar.md), where Block 2 is the fence and Block 3 is
the literal block.

This was a deliberate choice, re-confirmed when raised directly: error
codes reuse the phase1 analyzer's numbering (see
`docs/decisions/0006-phase1-block-numbering.syon`) so the two systems that
report on the same document — `phase1.report.syon` and a parse error —
agree with each other, rather than with `02-grammar.md`. A reader moving
between this table and the grammar spec must track which convention
applies where.

## Ranges

| Range | Meaning |
|---|---|
| 1-99 | Phase 1: general, pre-block errors |
| 101-199 | Block 1: records (mappings, sequences, scalars) |
| 201-299 | Block 2: literal blocks, `[[[ ... ]]]` |
| 301-399 | Block 3: document fences, ` ```path.format ` |

Within a range, the low two digits are reused across ranges for the "same
kind" of problem in a different block — e.g. `112`, `212`, `312` are all
"anchor (`&`) used where forbidden," just inside a record, a literal
block, and a fence body respectively.

## Table

| Code | Name | Meaning |
|---|---|---|
| SYON-001 | `INVALID_UTF8` | Input is not valid UTF-8. Reserved: callers are expected to reject this before invoking the parser. |
| SYON-002 | `TAB_IN_INDENTATION` | A tab character appears in a line's leading whitespace. |
| SYON-003 | `UNEXPECTED_TRAILING_CONTENT` | Reserved for trailing content after a complete document that no block rule accounts for. |
| SYON-090 | `MALFORMED_STRUCTURE` | Catch-all for a structural failure not covered by a more specific code (e.g. a pest grammar mismatch, or a Go type-coercion error during `Unmarshal`). |
| SYON-101 | `KEY_STARTS_WITH_OPERATOR` | A mapping key starts with `:`, `-`, or `#`. |
| SYON-102 | `EMPTY_KEY` | A mapping key is empty. |
| SYON-103 | `DUPLICATE_KEY` | The same key appears twice in one mapping. |
| SYON-111 | `EXPLICIT_TAG` | YAML explicit tag `!` / `!!` used in a record. |
| SYON-112 | `ANCHOR` | YAML anchor `&name` used in a record. |
| SYON-113 | `ALIAS` | YAML alias `*name` used in a record. |
| SYON-114 | `FLOW_MAPPING` | YAML flow mapping `{...}` used in a record. |
| SYON-115 | `FLOW_SEQUENCE` | YAML flow sequence `[...]` used in a record. |
| SYON-116 | `COMPLEX_KEY` | YAML complex key `? ...` used in a record. |
| SYON-117 | `DOCUMENT_START_MARKER` | YAML `---` document-start marker used. |
| SYON-118 | `DOCUMENT_END_MARKER` | YAML `...` document-end marker used. |
| SYON-121 | `UNTERMINATED_QUOTED_STRING` | A double-quoted string is opened but never closed. |
| SYON-202 | `UNTERMINATED_LITERAL_BLOCK` | A `[[[` literal block is opened but never closed with a matching `]]]`. |
| SYON-211 | `LITERAL_EXPLICIT_TAG` | Reserved, not reachable: literal block bodies are opaque, verbatim text and are never scanned for forbidden constructs. |
| SYON-212 | `LITERAL_ANCHOR` | Reserved, not reachable (see 211). |
| SYON-213 | `LITERAL_ALIAS` | Reserved, not reachable (see 211). |
| SYON-214 | `LITERAL_FLOW_MAPPING` | Reserved, not reachable (see 211). |
| SYON-215 | `LITERAL_FLOW_SEQUENCE` | Reserved, not reachable (see 211). |
| SYON-216 | `LITERAL_COMPLEX_KEY` | Reserved, not reachable (see 211). |
| SYON-217 | `LITERAL_DOCUMENT_START_MARKER` | Reserved, not reachable (see 211). |
| SYON-218 | `LITERAL_DOCUMENT_END_MARKER` | Reserved, not reachable (see 211). |
| SYON-301 | `FENCE_INFO_STRING_MALFORMED` | A ` ``` ` fence's info string is not `path.format` (missing the `.`). |
| SYON-302 | `UNTERMINATED_FENCE` | A ` ``` ` fence is opened but never closed with a matching ` ``` `. |
| SYON-311 | `FENCE_EXPLICIT_TAG` | YAML explicit tag found inside fence content. **Rust only** — see "Per-implementation reachability" below. |
| SYON-312 | `FENCE_ANCHOR` | YAML anchor found inside fence content. Rust only. |
| SYON-313 | `FENCE_ALIAS` | YAML alias found inside fence content. Rust only. |
| SYON-314 | `FENCE_FLOW_MAPPING` | Flow mapping found inside fence content. Rust only. |
| SYON-315 | `FENCE_FLOW_SEQUENCE` | Flow sequence found inside fence content. Rust only. |
| SYON-316 | `FENCE_COMPLEX_KEY` | Complex key found inside fence content. Rust only. |
| SYON-317 | `FENCE_DOCUMENT_START_MARKER` | `---` found inside fence content. Rust only. |
| SYON-318 | `FENCE_DOCUMENT_END_MARKER` | `...` found inside fence content. Rust only. |

## Per-implementation reachability

The three implementations do not detect identical error conditions, because
their architectures differ (see
`docs/decisions/0004-independent-go-implementation.syon`):

- **211-218** (forbidden construct inside a literal block) are not
  reachable in any implementation: `[[[ ... ]]]` bodies are always treated
  as opaque, verbatim text.
- **311-318** (forbidden construct inside fence content) are reachable in
  **Rust only**. Rust's preflight scan does not exempt fence bodies from
  the same forbidden-construct scan applied to records, so a fence
  containing e.g. embedded YAML with an anchor is rejected. Go's fence
  handling instead treats fence content as opaque, so these codes are
  reserved (unreachable) there. This is a known, tracked divergence, not a
  design goal — see `docs/decisions/0006-phase1-block-numbering.syon` and
  `docs/decisions/0007-parse-error-codes.syon`, since
  [03-semantics.md](03-semantics.md) states fence content is returned as a
  raw string and never parsed.
- **SYON-001**, **SYON-003**, **SYON-118** are reserved in the current Go
  implementation: it does not construct these codes today (invalid UTF-8 is
  expected to be rejected by the caller before `Unmarshal`/`Parse` is
  invoked; `...` document-end markers are not currently checked by Go).

## Per-language surface

- **Rust**: `syon_parser::ErrorCode`, an `enum` with explicit discriminants
  matching the table above; `SyonError::code() -> ErrorCode`. `Display`
  renders as `SYON-{:03}`.
- **Go**: `syon.ErrorCode`, an `int`-based type with named constants
  (`syon.CodeAnchor`, etc.); `syon.Error.Code`. `String()` renders as
  `SYON-%03d`.
- **Python**: `syon.ErrorCode`, a `pyclass` enum exposed with Python-style
  `UPPER_CASE` member names (`syon.ErrorCode.ANCHOR`), independent of the
  Rust crate's PascalCase Rust identifiers — see
  `docs/decisions/0007-parse-error-codes.syon` for why this is a hand-kept
  mirror rather than a generated binding. Parse errors are raised as
  `syon.SyonError`, a `ValueError` subclass carrying `.code` (a
  `syon.ErrorCode`) and `.message` (`str`).
