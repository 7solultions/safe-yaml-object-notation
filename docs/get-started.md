# Get started

## Prerequisites

- A recent stable [Rust toolchain](https://www.rust-lang.org/tools/install)
  (edition 2021).
- [`task`](https://taskfile.dev) (optional but recommended — the repository
  ships a `Taskfile.yml` with all the common commands).

## Build the workspace

```bash
cargo build --workspace
```

or, using the task runner:

```bash
task build-all-crates
```

## Parse your first file

The `syon-cli` crate builds a `syon` binary that parses a `.syon` file and
prints its AST as JSON:

```bash
task build-parser-crate
task run-cli-binary -- examples/glossary/entries/syon.syon
```

which is equivalent to:

```bash
cargo run -p syon-cli -- examples/glossary/entries/syon.syon
```

## Run the tests

```bash
task test-all-crates
# or, just the parser crate:
task test-parser-crate
```

## Other useful tasks

| Task | What it does |
|------|---------------|
| `task check-all-crates` | Type-check the whole workspace without producing artifacts |
| `task lint-all-crates` | Run Clippy with warnings denied |
| `task format-all-crates` | Format every source file with `rustfmt` |
| `task clean-build-artifacts` | Remove all Cargo build artifacts |
| `task build-python-bindings` | Build and install the `syon` Python extension in development mode (requires [maturin](https://www.maturin.rs/)) |
| `task build-ffi-library` | Build the `syon-parser` `cdylib`/`staticlib` in release mode |

See [`Taskfile.yml`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/Taskfile.yml)
for the full list.

## Next steps

Continue with the [language guide](language.md) to learn the SYON data model,
or jump to the [CLI reference](cli.md) and [Python bindings](python.md) pages.
