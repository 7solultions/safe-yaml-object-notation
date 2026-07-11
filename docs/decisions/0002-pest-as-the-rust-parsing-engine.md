# 2. Use pest as the Rust parsing engine

Date: 2026-07-11

## Status

Accepted

## Context

`syon-parser` went through three parsing approaches before settling:

1. **winnow** (initial scaffold) — a hand-written combinator parser over
   the lexer's token stream.
2. **saphyr-parser 0.0.6** — a YAML 1.2 *event* parser, on the theory that
   SYON's Block 1 is a YAML block-style subset, so an existing YAML event
   stream could be filtered to reject the forbidden construct set (this is
   still the strategy described in `spec/02-grammar.md`). This required a
   separate preflight text scan alongside it, because saphyr-parser's
   event stream doesn't carry block-vs-flow style information — the
   events for `key: [a, b]` and a hypothetical block-style equivalent look
   the same, so flow collections couldn't be rejected from the event
   stream alone.
3. **pest** (current) — a native PEG grammar
   (`crates/syon-parser/src/grammar.pest`) encoding SYON's structure and
   spacing rule directly, with an indentation-aware builder walking the
   flat pest output into the typed AST.

## Decision

Use pest as the parsing engine, with SYON's grammar defined natively in
`grammar.pest` rather than adapting a general-purpose YAML parser.

Reusing a YAML parser (saphyr-parser) seemed attractive since Block 1 is
a YAML subset, but SYON's spacing rule (`:`/`-`/`#` are structural only
when followed by whitespace) and its non-YAML constructs (Block 2 fences,
Block 3 literal blocks) don't map onto YAML's event model cleanly. A
native PEG grammar can express all three block types and the spacing rule
directly, without fighting an external parser's assumptions.

## Consequences

- The grammar is fully owned and auditable in one file
  (`grammar.pest`) rather than depending on a YAML parser's interpretation
  of "flow style" or "block style".
- `spec/02-grammar.md`'s note that "the recommended implementation
  strategy is to use `saphyr-parser`" is now stale — it describes the
  rejected second approach, not the current implementation. It should be
  updated or removed.
- The preflight text-scan pattern (see ADR 0003) was introduced to work
  around saphyr-parser's lack of style information, but was *kept* after
  switching to pest rather than folded entirely into the grammar — see
  ADR 0003 for why, and for a real bug that pattern caused.
- `syon-go` (ADR 0004) is an independent implementation and does not use
  pest or any parsing library — it has its own hand-written recursive
  descent parser, so this decision only binds the Rust implementation.
