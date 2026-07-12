# CLI reference

The `syon-cli` crate builds a single binary, `syon`, that parses a `.syon`
file and prints its AST as JSON.

## Usage

```text
syon <file.syon>
```

- Reads `<file.syon>`, parses it, and prints the parsed body of the first
  document as pretty-printed JSON to stdout.
- On a read or parse error, prints the error to stderr and exits with a
  non-zero status.

## Example

```bash
cargo run -p syon-cli -- examples/glossary/entries/syon.syon
```

Given the [glossary entry example](examples.md), this prints the parsed
mapping as JSON, e.g.:

```json
{
  "abbreviation": "SYON",
  "term": "Safe YAML Object Notation",
  "id": "syon-001",
  "version": "0.9.0",
  "description": "A human-writable data serialization format...",
  "contexts": [
    "data-formats",
    "serialization",
    "one-family"
  ]
}
```

All values are JSON strings, arrays, and objects — SYON's
[strings-only boundary](language.md#strings-only-boundary) means no implicit
number or boolean coercion happens here either.

## Building a release binary

```bash
cargo build -p syon-cli --release
```

The binary is placed at `target/release/syon`.

## `syon phase1` — usage evaluation

```text
syon phase1 [FILE...]
```

Evaluates each file's use of Block 1 (records, including the spacing-rule
symbols `: `, `- `, `# ` and their inline/literal use elsewhere), Block 2
(`[[[ ... ]]]` literal blocks — this subcommand's own numbering, the
opposite of the grammar spec's; see
[ADR 0006](decisions/0006-phase1-block-numbering.syon)), and Block 3
(` ``` `path.format` `` ` document fences), then writes a `phase1.report.syon`
with a complexity score and a YAML 1.2 compatibility estimate per file plus a
corpus-wide summary.

With no file arguments, it walks the default corpus: `examples/**/*.syon`
and `docs/decisions/*.syon`.

```bash
task phase1-report          # Rust — writes ./phase1.report.syon
task phase1-report-go       # Go   — writes ./syon-go/phase1.report.syon
```

The report is itself a SYON document (see the
[decisions record schema](decisions/README.md#record-schema) for the general
style), parseable by both implementations. Complexity weights and the
compatibility formula are documented directly in
[`crates/syon-parser/src/phase1.rs`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/crates/syon-parser/src/phase1.rs).
