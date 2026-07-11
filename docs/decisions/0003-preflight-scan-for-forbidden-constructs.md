# 3. Preflight text scan for forbidden constructs

Date: 2026-07-11

## Status

Accepted

## Context

SYON must reject a set of YAML constructs (tags, anchors/aliases, flow
collections, `?` complex keys, `---`/`...` document markers). In
`syon-parser`, this rejection happens in two layers:

- `grammar.pest` defines `forbidden_*` rules, but per the file's own
  comment they are "used in error-reporting; the parser rejects them"
  elsewhere.
- `preflight()` in `parser.rs` does the actual rejection: a line-by-line
  scan of the *raw source text*, before pest parsing runs at all, looking
  for these constructs outside double-quoted strings.

This split dates back to the saphyr-parser era (ADR 0002): saphyr's YAML
event stream didn't carry block-vs-flow style, so flow collections
couldn't be rejected from the event stream and a text-level preflight
scan was added to cover that gap. When the parser was rewritten around
pest, the preflight scan was kept rather than folded entirely into the
grammar.

`syon-go` does not have this split — its forbidden-construct checks live
inline in the recursive-descent parser itself, checked at the point each
construct would otherwise be consumed.

## Decision

Keep a preflight text scan as a distinct pass ahead of grammar-based
parsing in the Rust implementation, rather than expressing all
forbidden-construct rejection inside `grammar.pest` alone.

## Consequences

- The preflight scan and the grammar must independently agree on what
  counts as, e.g., a literal-block opener versus a flow sequence. They
  are two separate implementations of "is this position a value
  position", and can drift out of sync.
- This already happened once and caused a real bug: the preflight scan's
  `[[[`/`]]]` recognition only handled the delimiter alone on its own
  line, not `key: [[[` (the delimiter opening a literal block on the same
  line as its key) — even though the grammar and AST layer already
  supported that form. Every document using that form (including the
  project's own canonical examples in `examples/glossary/`) was rejected
  as "forbidden: flow collection `[`" until this was fixed. See the
  `preflight()`/`grammar.pest` changes in PR #6.
- Any future change to what a "value position" means (new block type, new
  spacing-rule nuance) needs to be made in both `preflight()` and
  `grammar.pest`, and needs a test exercising both layers together — a
  test on the grammar alone, or the preflight scan alone, would not have
  caught the bug above.
- If this class of bug recurs, the alternative — expressing forbidden
  constructs entirely as pest grammar rules with no separate text scan —
  should be reconsidered as a superseding ADR.
