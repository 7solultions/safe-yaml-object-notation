# SYON — Safe YAML Object Notation

SYON is a YAML-inspired, minimal object-notation language designed for safety and predictability.
It supports the core data model of YAML — scalars, sequences, and mappings — while deliberately
excluding anchors, aliases, arbitrary tags, and multi-document streams.

## Goals

- **Safe**: no executable directives, no reference cycles, no arbitrary type coercion.
- **Readable**: indentation-based, human-friendly syntax.
- **Embeddable**: a single Rust library crate with no unsafe code.

## Workspace layout

```
crates/
  syon-parser/   # tokenizer + winnow-based parser, produces an AST
  syon-cli/      # `syon` binary — parses a .syon file and prints the AST as JSON
spec/            # language specification
```

## Quick start

```bash
task build-parser-crate
task run-cli-binary -- examples/hello.syon
```

## Spec

See [`spec/README.md`](spec/README.md) for the full language specification.

## Documentation

The [`docs/`](docs) directory holds a [Zensical](https://zensical.org)
documentation site (getting started, language guide, CLI reference, Python
bindings, and the glossary example). Build it with:

```bash
pip install zensical
task docs-serve   # local live-reload preview at http://localhost:8000
task docs-build   # static site in site/
```

On push to `main`, [`.github/workflows/docs.yml`](.github/workflows/docs.yml)
builds and publishes the site to GitHub Pages at
<https://object-notation-environment.github.io/safe-yaml-object-notation/>.
This requires enabling Pages once, in the repo's **Settings → Pages**, with
**Source** set to **GitHub Actions**.

## License

MIT
