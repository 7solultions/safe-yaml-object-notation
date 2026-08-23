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

## Errors

On a parse error, `syon.parse` raises `syon.SyonError`, an `Exception`
subclass carrying the failure in three parts:

```python
try:
    syon.parse("```path.json\nkey: value\n")
except syon.SyonError as e:
    e.code      # <ErrorCode.UNTERMINATED_FENCE: 202>
    int(e.code) # 202
    e.kind      # "syntax"  (or "forbidden")
    e.message   # 'line 1: unterminated ``` document fence'
    str(e)      # '[SYON-202] syntax error: line 1: unterminated ``` document fence'
```

Match on `e.code` rather than on the message text — the code is stable API and
the wording is not. The constants live on `syon.ErrorCode`:

```python
if e.code == syon.ErrorCode.DUPLICATE_KEY:
    ...
```

`kind` keeps the `forbidden` / `syntax` distinction from the [error
model](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/spec/03-semantics.md):
`forbidden` means SYON rejected a YAML construct on purpose, `syntax` means
the input was malformed. See
[`spec/05-error-codes.md`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/spec/05-error-codes.md)
for the full code table.
