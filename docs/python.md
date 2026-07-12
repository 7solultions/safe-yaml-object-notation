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

## Testing

Two test suites live in `crates/syon-python/`, both requiring the extension
to be built first:

```bash
task build-python-bindings
pip install "crates/syon-python[test]"

task test-python       # pytest: crates/syon-python/tests/
task test-python-bdd   # behave: crates/syon-python/features/
```

`tests/test_parse.py` is a plain [pytest](https://pytest.org) suite covering
parsing, the strings-only boundary, forbidden-construct errors, and every
file under `examples/` and `docs/decisions/` (mirroring the Rust and Go
corpus checks in CI).

`features/spacing_rule.feature` is a small [behave](https://behave.readthedocs.io)
(Gherkin/BDD) suite specifically translating
[the spacing rule](language.md#block-1-record-yaml-block-style-subset)'s
examples from `spec/01-lexer.md` into executable `Given`/`When`/`Then`
scenarios — a readable, spec-as-tests companion to the pytest suite, not a
replacement for it.
