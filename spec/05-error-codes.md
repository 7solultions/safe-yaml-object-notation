# Error codes

Every `SyonError` (Rust), `syon.Error` (Go), and `syon.SyonError` (Python)
carries a three-digit numeric code alongside its human-readable message, in
addition to the `Forbidden`/`Syntax` distinction from
[03-semantics.md](03-semantics.md)'s error model.

The code is API; the message wording is not. Match on the code to ask "is this
specifically a duplicate-key error?" rather than string-matching the message,
which is fragile across rewordings and impossible to keep in sync across three
independently-written parsers (see ADR 0004).

## Ranges

| Range | Meaning |
|---|---|
| 1-99 | General — whole-file, or outside the block model entirely |
| 11-19 | Decode-time, after a successful parse |
| 101-199 | Block 1: records (mappings, sequences, scalars, `\|` block scalars) |
| 201-299 | Block 2: document fences, ` ```path.format ` |

The banding follows [02-grammar.md](02-grammar.md)'s two-block model. The
phase1 analyzer uses the same numbering: the two schemes disagreed only while
the `[[[ ... ]]]` escape hatch existed, and ADR 0007 removed it, which left
them nothing to disagree about (see ADR 0006).

Within a range, the low two digits are reserved for the "same kind" of problem
in a different block — `112` and `212` would both be "anchor (`&`) used where
forbidden", in a record and in a fence body respectively. Only the `1xx` forms
are currently produced; the `2xx` forms are held for when fence bodies are
scanned separately from the document that contains them.

## Table

Reachability differs by implementation, deliberately: Rust carries a leading
`[` or `{` through as scalar text because a pre-parse scan cannot tell an
indicator from ordinary content, where Go rejects it outright. A code marked
**reserved** is defined in all three enums, so they stay in lockstep, but is
not currently produced anywhere.

| Code | Name | Meaning | Produced by |
|---|---|---|---|
| SYON-001 | `INVALID_UTF8` | Input is not valid UTF-8. | reserved — callers reject this before parsing |
| SYON-002 | `TAB_IN_INDENTATION` | A tab character appears in a line's leading whitespace. | Rust, Go |
| SYON-003 | `UNEXPECTED_CONTENT` | A line was claimed by no construct — neither a mapping entry, a sequence item, nor the body of a `\|` block. | Rust, Go |
| SYON-004 | `INDENT_NOT_MULTIPLE_OF_STEP` | Indentation is not a multiple of the configured step. | Rust (Go's step is not configurable) |
| SYON-011 | `DECODE_TYPE_MISMATCH` | A scalar could not be decoded into the requested target type. | Go (`Unmarshal`) |
| SYON-012 | `DECODE_SHAPE_MISMATCH` | A node's shape did not match the target type — a mapping wanted where a sequence stood, or vice versa. | Go (`Unmarshal`) |
| SYON-090 | `MALFORMED_STRUCTURE` | Catch-all for a grammar failure with no more specific code. | Rust (a raw pest error) |
| SYON-101 | `KEY_STARTS_WITH_OPERATOR` | A mapping key starts with `:`, `-`, or `#`. | Rust, Go |
| SYON-102 | `EMPTY_KEY` | A mapping key is empty. | Go |
| SYON-103 | `DUPLICATE_KEY` | The same key appears twice in one mapping. | Rust, Go |
| SYON-111 | `EXPLICIT_TAG` | YAML explicit tag `!` / `!!`. | Rust, Go |
| SYON-112 | `ANCHOR` | YAML anchor `&name`. | Rust, Go |
| SYON-113 | `ALIAS` | YAML alias `*name`. | Rust, Go |
| SYON-114 | `FLOW_MAPPING` | YAML flow mapping `{...}`. | Go (Rust reads it as scalar text) |
| SYON-115 | `FLOW_SEQUENCE` | YAML flow sequence `[...]`. | Go (Rust reads it as scalar text) |
| SYON-116 | `COMPLEX_KEY` | YAML complex key `? ...`. | Rust, Go |
| SYON-117 | `DOCUMENT_START_MARKER` | A `---` marker after content has begun. A single leading `---` is allowed: it opens the one document the file holds. | Rust, Go |
| SYON-118 | `DOCUMENT_END_MARKER` | A `...` document-end marker. | Rust |
| SYON-119 | `LITERAL_BLOCK_REMOVED` | `[[[` or `]]]`, the escape hatch removed by ADR 0007. Named explicitly so the message can point at `\|` instead of falling through to a generic flow-sequence error. | Rust, Go |
| SYON-121 | `UNTERMINATED_QUOTED_STRING` | A quoted scalar is opened but never closed. | Go (in Rust this is SYON-090) |
| SYON-131 | `COMPACT_BLOCK_SCALAR_NEEDS_OPTION` | A sequence item writes `- key: \|` without `allow_key_in_line_after_list`. | Rust (the option is Rust-side) |
| SYON-132 | `SEQUENCE_ITEM_MIXES_MAPPING_AND_BLOCK` | A sequence item mixes `key: value` with an indented non-mapping block. | Rust |
| SYON-133 | `SEQUENCE_ITEM_INLINE_TEXT_AND_BLOCK` | A sequence item has both inline text and an indented block. | Rust |
| SYON-201 | `FENCE_INFO_STRING_MALFORMED` | A fence info string is missing its `path.format` separator. | Go (in Rust the line fails to match `fence_open` and becomes SYON-003) |
| SYON-202 | `UNTERMINATED_FENCE` | A ` ``` ` fence is opened but never closed. | Rust, Go |

## Where the table lives

The code for each implementation is the source of truth for its own
reachability, and each pins the codes in a test so a renumbering cannot land
silently:

| Implementation | Definition | Test |
|---|---|---|
| Rust | `crates/syon-parser/src/error_code.rs` | `error_codes_are_stable` in `parser.rs` |
| Go | `syon-go/error_code.go` | `TestForbiddenAndSyntax` in `syon_test.go` |
| Python | `crates/syon-python/src/lib.rs` (`ErrorCode`) | `test_error_code_is_stable` in `tests/test_syon.py` |

## Rendering

Rust's `Display`, Go's `Error()`, and Python's `__str__` all prefix the code:

```text
[SYON-202] syntax error: line 1: unterminated ``` document fence
```

Python additionally exposes the parts separately:

```python
try:
    syon.parse("```path.json\nkey: value\n")
except syon.SyonError as e:
    e.code      # <ErrorCode.UNTERMINATED_FENCE: 202>
    int(e.code) # 202
    e.kind      # "syntax"
    e.message   # 'line 1: unterminated ``` document fence'
```

## Adding a code

Add the variant to all three enums in the same change, with the same number,
and add a row here. The Python `From<ErrorCode>` impl is exhaustive on
purpose: adding a Rust variant without mirroring it is a compile error, not a
silently missing constant.
