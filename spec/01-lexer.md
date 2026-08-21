# SYON Lexer Specification (v0.9.0)

## Encoding

SYON source files MUST be UTF-8 encoded. A file that is not valid UTF-8 MUST
be rejected with a decode error before any lexing begins.

## Line orientation

SYON is line-oriented. Lines are separated by `\n` (LF) or `\r\n` (CRLF);
both are normalised to `\n`. Trailing whitespace on any line is ignored for
structural purposes.

## Indentation

- **Spaces only.** Tabs in the indentation prefix are a lexer error.
- **No trailing tabs.** Trailing whitespace (spaces or tabs) is discarded.
- Blank lines (empty or whitespace-only) are skipped; they do not affect
  indentation tracking.
- An increase in leading spaces relative to the previous non-blank line emits
  an `Indent` token; a decrease emits a `Dedent` token.

## The spacing rule (Section 2.4)

A character is **structural** only when it is followed by a space (`U+0020`),
a tab, or an end-of-line (EOL), and only in the following specific position:

| Marker | Structural position | Literal form |
|--------|---------------------|--------------|
| `:`    | The **first** `: `, `:\n`, or `:<EOL>` on the line, ending a mapping key | `:x` (no space follows); any *later* `:` on the same line, even `: `-shaped |
| `-`    | The **first non-space character** on the line, starting a sequence item | `-x` or `-1`; any `-` that is not the first non-space character on the line, even if space-adjacent |
| `#`    | At line-start (after only whitespace), or after a space anywhere later on the line, starting a comment | `#x` or `abc#123` |

This means values do **not** need quoting or escaping for these characters as
long as they are not in the specific structural position for their marker.
Two consequences of this are easy to get wrong and worth stating explicitly:

- **Only the first colon-space on a line is structural.** Once a key's `: `
  has been consumed, every subsequent `:` on that same line — even one
  followed by a space — is ordinary value text, never re-interpreted as
  another key separator:

  ```
  key: value: with a colon: and another
  ```

  parses as one mapping entry, `key` → the scalar `value: with a colon: and
  another`.

- **A dash is a sequence-item marker only as the first non-space character
  of the line.** A `-` occurring later in the line — even preceded by a
  space, even followed by a space — is ordinary value text, not a nested
  list item:

  ```
  note: this - is not a list item
  a-b: still just a key named "a-b"
  ```

  Neither `-` above is structural. Only a `-` immediately after a line's
  indentation, as in

  ```
  seq:
    - this is a list item
  ```

  is a sequence-item marker.

Examples:

```
url: https://example.com   # ok — `:` in the value is not followed by space
tag: -draft                # ok — `-` in the value is not the first non-space character
id:  abc#123               # ok — `#` is not preceded by space
```

## Token types

| Token | Description |
|-------|-------------|
| `Key(String)` | Bare key preceding a structural `: ` |
| `Value(String)` | Scalar value on the same line as a key or list item |
| `ListItem` | The `- ` sequence item marker |
| `Comment(String)` | Text following a structural `# ` marker |
| `BlockHeader(chomp)` | A `|` (or `>`) block-scalar header in a value position |
| `DocFence { path, format }` | Opening triple-backtick fence with `path.format` info string |
| `Indent` | Indentation level increased |
| `Dedent` | Indentation level decreased |

## Key restrictions

Keys MUST NOT begin with any operator symbol (`:`, `-`, `#`). A key such as
`:bad` or `-also-bad` is a lexer error.
