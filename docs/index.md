# SYON — Safe YAML Object Notation

SYON is a YAML-inspired, minimal object-notation language designed for safety
and predictability. It supports the core data model of YAML — scalars,
sequences, and mappings — while deliberately excluding anchors, aliases,
arbitrary tags, and multi-document streams.

Every valid SYON document is a valid YAML document, but not every valid YAML
document is a valid SYON document: SYON is a strict, safe subset.

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
