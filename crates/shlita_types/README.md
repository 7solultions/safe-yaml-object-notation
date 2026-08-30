# shlita_types

The lower of the two shlita crates: IEC 61131-3's **data types** and its
**standard functions**. Nothing here has state. The timers, counters,
bistables and edge detectors do, and they live in `shlita_runtime`.

| Item | What it is |
|------|------------|
| `ElementaryType` | The 27 elementary types of the third edition — `BOOL`, `SINT`…`LINT`, `USINT`…`ULINT`, `BYTE`…`LWORD`, `REAL`/`LREAL`, `TIME`/`LTIME`, the three date types in both widths, `CHAR`/`WCHAR`, `STRING`/`WSTRING`. |
| `ElementaryValue` | A value, with its type attached. An `INT` and a `DINT` holding 7 are not equal. |
| `ElementaryClass` | The leaves of the standard's generic hierarchy; `is_any_int`, `is_any_bit` and the rest are the interior nodes. |
| `StandardFunction` | The 47 stateless functions: bitwise, shifts, selection, comparison, arithmetic, numeric and string. |
| `convert` | The explicit `*_TO_*` conversions, and the bridge to sheni's primitives. |
| `ShlitaError` / `ShlitaCode` | Why a literal or a call failed, with a stable numeric code in the `701-799` band. |

## Reading a literal

```rust
use shlita_types::{ElementaryType, ShlitaCode};

assert_eq!(ElementaryType::Int.read("16#7FFF").unwrap().to_string(), "32767");
assert_eq!(ElementaryType::Word.read("2#1010_1010").unwrap().to_string(), "16#00AA");
assert_eq!(ElementaryType::Time.read("T#1d2h3m4s5ms").unwrap().to_string(), "T#1d2h3m4s5ms");

// It does not wrap when it does not fit.
assert_eq!(
    ElementaryType::Sint.read("128").unwrap_err().code(),
    ShlitaCode::IntegerOutOfRange
);
```

Canonical text reads back: printing a value and reading the result gives an
equal value, at every type.

## Why its own vocabulary

Three of the standard's types have no honest counterpart in sheni, so the
crate defines its own and offers conversions instead of reusing sheni's
primitives:

- `BYTE` is an eight-bit **string** that may be `AND`ed. Sheni's `byte` is a
  character.
- The bit strings are types distinct from the unsigned integers of the same
  width, and the standard keeps them apart precisely so that `AND(UINT, UINT)`
  can be an error.
- `TIME` admits a sign, a fraction on its least significant unit, an
  underscore between groups, overflow in its leading unit and a `TIME#` long
  form. Sheni's `duration` refuses all five.

```rust
use shlita_types::{call, ElementaryType, ShlitaCode};

let a = ElementaryType::Byte.read("2#1100").unwrap();
let b = ElementaryType::Byte.read("2#1010").unwrap();
assert_eq!(call("AND", &[a, b]).unwrap().to_string(), "16#08");

let n = ElementaryType::Usint.read("12").unwrap();
assert_eq!(
    call("AND", &[n.clone(), n]).unwrap_err().code(),
    ShlitaCode::NotABitString
);
```

## Calling a function

Arguments agree in type — the standard defines no `ADD(INT, DINT)` — and a
result that does not fit is reported rather than wrapped.

```rust
use shlita_types::{call, ElementaryType, ShlitaCode};

let span = ElementaryType::Time.read("T#1s").unwrap();
let three = ElementaryType::Int.read("3").unwrap();
assert_eq!(call("MUL", &[span, three]).unwrap().to_string(), "T#3s");

let big = ElementaryType::Sint.read("127").unwrap();
let one = ElementaryType::Sint.read("1").unwrap();
assert_eq!(
    call("ADD", &[big, one]).unwrap_err().code(),
    ShlitaCode::ArithmeticOverflow
);
```

## Decisions

- [`ADR_shlita_01`](../../design/architecture/ADR_shlita_01__two_crates_and_scope.syon) — the standard splits into a type vocabulary and a scan runtime, and neither one is a language
