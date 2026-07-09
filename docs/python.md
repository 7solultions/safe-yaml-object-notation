# Python bindings

The `syon-python` crate exposes the `syon_parser` crate to Python via
[PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs), as a module
named `syon`.

## Building

```bash
task build-python-bindings
```

which runs:

```bash
maturin develop -m crates/syon-python/Cargo.toml
```

This builds the extension module and installs it into your active Python
environment (a virtualenv is recommended).

## Usage

```python
import syon

value = syon.parse("""
name: Alice
age: 30
contexts:
  - data-formats
  - serialization
""")

print(value)
# {'name': 'Alice', 'age': '30', 'contexts': ['data-formats', 'serialization']}
```

`syon.parse` takes a SYON source string and returns the parsed body of the
first document, converted to native Python types:

| SYON `Value` | Python type |
|--------------|-------------|
| `Scalar` | `str` |
| `LiteralBlock` | `str` |
| `Mapping` | `dict` |
| `Sequence` | `list` |

As with the [Rust API](language.md#strings-only-boundary), every scalar comes
back as a `str` — SYON does not guess that `"30"` should be an `int`.

On a parse error, `syon.parse` raises `ValueError` with the underlying
`SyonError` message.
