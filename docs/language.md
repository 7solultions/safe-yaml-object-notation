# Language guide

SYON documents are built from three block types, which can appear at any
nesting level. The full normative rules live in the [specification](spec/01-lexer.md);
this page is a practical overview.

## Block 1 — Record (YAML block-style subset)

The primary block type: indentation-based mappings and sequences, using the
three structural markers `: `, `- `, `# `. A marker is only *structural* when
followed by a space or end-of-line — otherwise it's ordinary text, so values
rarely need quoting:

```syon
url: https://example.com   # ok — `:` in the value is not followed by a space
tag: -draft                # ok — `-` here isn't a list marker
id:  abc#123               # ok — `#` isn't preceded by whitespace
```

Sequences and mappings nest via indentation (spaces only — tabs in an
indentation prefix are a lexer error):

```syon
name: Alice
age: 30
contexts:
  - data-formats
  - serialization
```

## Block 2 — Document fence

An embedded sub-document with an explicit media-type annotation, delimited by
two triple-backtick fences at column 0. The info string on the opening fence
is `path.format`; SYON does not parse the fenced content itself, it's
returned verbatim in the AST for the application to dispatch:

````syon
```config/settings.json
{ "embedded": true }
```
````

## Block 3 — Literal escape hatch

A verbatim, uninterpreted block delimited by `[[[` and `]]]`, useful for
multi-line prose or content that would otherwise need heavy escaping:

```syon
description: [[[
  A human-writable data serialization format that is safe, simple,
  and structured.
]]]
```

## Safety: what's forbidden

A conforming SYON parser rejects the following YAML constructs:

| Construct | Why forbidden |
|-----------|---------------|
| `!tag` / `!!type` explicit tags | Introduce arbitrary typing |
| `&anchor` / `*alias` | Enable reference cycles |
| `{…}` flow mappings / `[…]` flow sequences | Disallowed flow style |
| `?` complex key | Not needed in the safe subset |
| `---` / `...` document markers | No multi-document streams |
| Duplicate keys in a mapping | Ambiguous — always a parse error |

## Strings-only boundary

All scalars are strings at the parse boundary — SYON never guesses a type
for you:

| Input token | SYON value |
|-------------|------------|
| `42` | `Scalar("42")` — not an integer |
| `true` | `Scalar("true")` — not a boolean |
| `null` | `Scalar("null")` — not null |
| `"hello"` | `Scalar("hello")` — quotes stripped |
| `[[[…]]]` | `LiteralBlock(…)` — verbatim string |

Applications that need typed values perform their own post-parse
interpretation.

## Comments are first-class

Comments are attached to the AST, not discarded:

1. A block of `# ` lines immediately before a key becomes that entry's
   `leading_comments`.
2. A `# ` fragment on the same line as a key or value becomes that entry's
   `trailing_comment`.
3. Anything else (trailing comments at the end of a document) is attached to
   the document.

## The AST

```text
Value
  ├── Scalar(String)
  ├── LiteralBlock(String)     verbatim [[[ … ]]] content
  ├── Mapping(Vec<MappingEntry>)
  │     MappingEntry { key, value, leading_comments, trailing_comment }
  └── Sequence(Vec<SequenceItem>)
        SequenceItem { value, leading_comments, trailing_comment }
```

A parsed file is a `SyonFile { documents: Vec<Document> }`, where each
`Document { path, format, body }` corresponds to one Block 2 fence (or the
implicit top-level document).

## Error model

There is no partial-result or best-effort mode: every parse either succeeds
with a complete `Document`, or fails with a `SyonError` that is either
`Forbidden` (a disallowed YAML construct) or `Syntax` (malformed input),
along with a 1-based line and column.

## Full specification

For the normative grammar, lexer rules, and semantics, see:

- [Lexer](spec/01-lexer.md)
- [Grammar](spec/02-grammar.md)
- [Semantics](spec/03-semantics.md)
- [Glossary schema convention](spec/04-glossary-schema.md)
