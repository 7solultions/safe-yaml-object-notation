# SYON — Safe YAML Object Notation

SYON is a YAML-inspired, minimal object-notation language designed for safety and predictability.
Its core record syntax supports the data model of YAML — scalars, sequences, and mappings —
while deliberately excluding anchors, aliases, and arbitrary tags. That core syntax alone is a
strict, safe subset of YAML; SYON as a whole is not, since it adds document fences and literal
blocks with no YAML equivalent. See [`spec/README.md`](spec/README.md#relationship-to-yaml) for
the full picture.

## Goals

- **Safe**: no executable directives, no reference cycles, no arbitrary type coercion.
- **Readable**: indentation-based, human-friendly syntax.
- **Embeddable**: a single Rust library crate with no unsafe code.

## Workspace layout

```
crates/
  syon-parser/   # pest-based parser, produces an AST
  syon-cli/      # `syon` binary — parses a .syon file and prints the AST as JSON
  syon-python/   # PyO3 bindings exposing syon-parser to Python
syon-go/         # independent, dependency-free Go implementation
spec/            # language specification
```

## Quick start

```bash
task build-parser
task run-cli-binary -- examples/glossary/entries/syon.syon
```

## Spec

See [`spec/README.md`](spec/README.md) for the full language specification.

## Architecture decisions

See [`docs/decisions/`](docs/decisions) for the ADR log — why pest was
chosen for the Rust parser, why Go got an independent implementation
instead of FFI bindings, and other architecturally significant calls.

## Roadmap / TODO

- [ ] Add a YAML-compatible mode for Block 2 (document fences), so fenced
      sub-documents can be parsed by a plain YAML 1.2 parser instead of
      requiring SYON-specific fence syntax.
      ([#2](https://github.com/object-notation-environment/safe-yaml-object-notation/issues/2))
- [x] Add a YAML-compatible mode for Block 3 (literal escape hatch), treating
      it as equivalent to a YAML multiline block scalar (`|`) rather than the
      SYON-specific `[[[`/`]]]` delimiters. Done, and further: `[[[` was
      removed outright rather than kept alongside `|`, so there is no mode to
      choose. See ADR 0007.
      ([#3](https://github.com/object-notation-environment/safe-yaml-object-notation/issues/3))

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
