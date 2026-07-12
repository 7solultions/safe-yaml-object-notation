# SYON Specification

This directory contains the normative specification for Safe YAML Object Notation (SYON).

| Document | Contents |
|---|---|
| [01-lexer.md](01-lexer.md) | Token types and lexical rules |
| [02-grammar.md](02-grammar.md) | Formal grammar (EBNF) |
| [03-semantics.md](03-semantics.md) | Value types, coercion rules, error model |
| [05-error-codes.md](05-error-codes.md) | Numeric error code table, per-language and per-implementation reachability |

## Relationship to YAML

Only SYON's Block 1 (record) syntax is a strict *safe subset* of YAML 1.2
block style:

- Every valid Block-1-only SYON document is a valid YAML document.
- Not every valid YAML document is a valid SYON document (see the forbidden
  set in `02-grammar.md`).

SYON as a whole is **not** a YAML subset. Block 2 (document fences) and
Block 3 (literal escape hatches) are SYON-specific syntax with no YAML
equivalent — a ` ```path.format ` fence or a `[[[`/`]]]` delimiter is not
valid YAML, so a document using either block type cannot be parsed by a
YAML 1.2 parser. In particular, SYON does not use YAML's native `---`/`...`
multi-document markers, but Block 2 fences do provide multi-document-style
embedding of arbitrary content through a different, SYON-only mechanism.

Excluded (Block 1 / YAML block style) features: anchors (`&`), aliases
(`*`), explicit tags (`!!`), directives (`%YAML`, `%TAG`), YAML's native
`---`/`...` document markers, YAML block scalars (`|`, `>`), flow
indicators used outside inline context.
