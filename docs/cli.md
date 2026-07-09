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
