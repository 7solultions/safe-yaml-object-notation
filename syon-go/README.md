# syon-go

A native **Go** parser for [SYON](https://github.com/object-notation-environment/safe-yaml-object-notation)
(Safe YAML Object Notation) — a safe, simple, structured relative of YAML.

No cgo, no dependencies. Drop-in-ish for `gopkg.in/yaml.v3`:

```go
import syon "github.com/object-notation-environment/syon-go"

var cfg Config
err := syon.Unmarshal(data, &cfg)   // or: node, err := syon.Parse(data)
```

## Why SYON

SYON keeps YAML's readable block style but removes the footguns:

- **Scalars are strings** — no implicit typing (`yes`/`no` don't become bools,
  `007` doesn't become octal). `Unmarshal` coerces to your struct field types.
- **Forbidden constructs rejected**: tags (`!x`/`!!x`), anchors/aliases
  (`&a`/`*a`), flow style (`{}`/`[]`), `?` complex keys, and `---` doc markers.
- **Two SYON blocks**: a `[[[ … ]]]` literal (verbatim multi-line text, dedented)
  and a ` ```path.format ` fenced document (embedded content, returned verbatim).
- **Duplicate keys are an error.**

## Mapping to Go

| SYON | Go |
|------|----|
| mapping | `struct` (by `syon:` tag, then `yaml:` tag, then lower-cased field), or `map[string]T` |
| sequence | `[]T` |
| scalar `"42"` | coerced to the field type: `int`, `bool`, `float`, `string` |
| `[[[ … ]]]` literal | `string` (dedented) |
| any | `map[string]any` / `[]any` / `string` |

The `yaml:` tag fallback means existing structs migrate from yaml.v3 with no tag
changes.

## Errors

Fatal, with a 1-based position and a kind — `forbidden` (a rejected YAML
construct) vs `syntax` (malformed input):

```
syon: forbidden error at line 3:6: flow sequences ([…]) are not allowed in SYON
```

## Writing

`Marshal` serializes a Go value back to SYON, round-tripping with `Unmarshal`:

```go
data, err := syon.Marshal(cfg)   // structs/maps → mappings, slices → sequences
```

Multi-line strings are written as `[[[ … ]]]` literals; empty slices/maps become a
bare `key:` (SYON has no `[]`/`{}` flow); values are quoted only when a bare scalar
would be misparsed.

## Status

v0.1 — reads **and writes**: parses Block 1 (records), Block 3 (literals) and
Block 2 (fences), enforces the safety rejections, decodes into Go values, and
serializes back. Comment-preservation in the AST is future work.

See the [SYON spec](https://github.com/object-notation-environment/safe-yaml-object-notation/tree/main/spec).
