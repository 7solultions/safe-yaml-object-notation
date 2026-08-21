# SYON — Safe YAML Object Notation

SYON is a YAML-inspired, minimal object-notation language designed for safety
and predictability. Its core record syntax supports the data model of YAML —
scalars, sequences, and mappings — while deliberately excluding anchors,
aliases, and arbitrary tags.

Only that core record syntax ([Block 1](language.md#block-1-record-yaml-block-style-subset))
is a strict, safe subset of YAML: every valid Block-1-only SYON document is
valid YAML, but not every valid YAML document is valid SYON. SYON as a whole
is **not** a YAML subset, though — it adds one SYON-specific block type with
no YAML equivalent: [document fences](language.md#block-2-document-fence)
for embedding sub-documents of any media type (SYON's alternative to YAML's
multi-document streams). Verbatim content uses a
[block scalar](language.md#block-scalars--verbatim-multi-line-text), which is
ordinary YAML. See [Relationship to YAML](language.md#relationship-to-yaml)
for the full picture.

## Goals

- **Safe** — no executable directives, no reference cycles, no arbitrary type
  coercion. Every scalar is a string at the parse boundary; there is no
  `yes`/`no`/`on`/`off` surprise and no accidental octal from a leading zero.
- **Readable** — indentation-based, human-friendly syntax with only three
  structural markers: `: `, `- `, `# `.
- **Embeddable** — a single Rust library crate ([`syon-parser`](language.md))
  with no unsafe code, plus a CLI ([`syon-cli`](cli.md)) and
  [Python bindings](python.md).

## Workspace layout

```text
crates/
  syon-parser/   # tokenizer + parser, produces an AST
  syon-cli/      # `syon` binary — parses a .syon file and prints the AST as JSON
  syon-python/   # PyO3 bindings exposing syon_parser to Python
spec/            # normative language specification
examples/        # sample .syon documents
```

## Where to go next

- [Get started](get-started.md) to build the workspace and parse your first file.
- [Language guide](language.md) for the three-block model and the safety
  rules that set SYON apart from full YAML.
- [CLI reference](cli.md) for the `syon` binary.
- [Python bindings](python.md) to call the parser from Python.
- [Glossary example](examples.md) for a worked example of using SYON as a
  schema-described data format.

## License

MIT
