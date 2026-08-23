# SYON Grammar (v0.9.0)

## Two-block model

A SYON document is composed of two distinct block types that can appear at
any nesting level.

An earlier revision had a third: `[[[ … ]]]`, a SYON-only literal escape
hatch. It was removed. A `|` block scalar does the same job in syntax a YAML
1.2 parser already understands, so keeping both bought nothing and cost the
Block 1 compatibility guarantee. `[[[` is now rejected by name.

### Block 1 — Record (YAML block-style subset)

The primary block type. Uses indentation and the structural markers `: `, `- `,
`# ` from the lexer.

```ebnf
record      = mapping | sequence | scalar ;
mapping     = { indent key ":" SP value newline } ;
sequence    = { indent "-" SP value newline } ;
scalar      = STRING | block-scalar ;   (* plain, double-quoted, or `|` *)
```

All content is a **string at the parse boundary** — no implicit type coercion
(see `03-semantics.md`).

#### Block scalars

Verbatim multi-line text uses YAML's block scalar: a `|` header in the value
position, with the body on the following lines, indented deeper than the key
or list item that owns it. The body is dedented by its common leading
indentation and is never parsed as SYON structure.

```
description: |
  any content, including YAML syntax characters
  and : - # markers that would otherwise be structural
```

The chomping indicators are YAML's: `|` keeps one trailing newline, `|-`
keeps none, `|+` keeps all of them.

`>` is accepted as a spelling of `|`. SYON has **no folded style** — `>` is
accepted so that YAML written for other tools keeps its meaning here rather
than silently folding newlines into spaces. A conforming parser MUST NOT
implement folding.

The parser returns the body as a `LiteralBlock(String)` value node.

The reference Rust implementation (`crates/syon-parser`) uses a native PEG
grammar (pest) encoding this structure and the spacing rule directly, paired
with a preflight text scan that rejects the forbidden construct set ahead of
grammar-based parsing — see ADR 0002 and ADR 0003 in `design/architecture/` for
why. An earlier revision of this spec recommended filtering a YAML 1.2 event
stream (e.g. `saphyr-parser`); that approach was tried and abandoned because
YAML event streams don't carry block-vs-flow style information, among other
mismatches with SYON's spacing rule.

### Block 2 — Document fence

An embedded sub-document with an explicit media-type annotation. The fence is
two triple-backtick lines at **column 0**, with a `path.format` info string on
the opening line.

```
```path/to/resource.json
{ … embedded content … }
```
```

The info string MUST contain at least one `.` separator; the part after the
last `.` is the format identifier. The parser exposes this as a `DocFence`
token with `path` and `format` fields.

## Forbidden set

The following YAML constructs MUST be rejected by a conforming SYON parser:

| Construct | Why forbidden |
|-----------|---------------|
| `!tag` / `!!type` explicit tags | Introduce arbitrary typing |
| `&anchor` anchors | Enable reference cycles |
| `*alias` aliases | Enable reference cycles |
| `{…}` flow mappings | Disallowed flow style |
| `[…]` flow sequences | Disallowed flow style |
| `,` as flow separator | Part of disallowed flow style |
| `?` complex key | Not needed in the safe subset |
| `---` explicit document-start marker | Superseded by Block 2 document fences (see above) |
| `...` document-end marker | Superseded by Block 2 document fences (see above) |
| `[[[` / `]]]` literal blocks | Removed from the language; use a `|` block scalar |

## Formal grammar (EBNF excerpt)

```ebnf
document       = block-1 | block-2 ;

(* Block 1 — YAML block style subset *)
block-1        = mapping | sequence | scalar ;
mapping        = mapping-entry { mapping-entry } ;
mapping-entry  = indent key COLON-SP value NEWLINE ;
sequence       = sequence-item { sequence-item } ;
sequence-item  = indent DASH-SP value NEWLINE ;
value          = scalar | mapping | sequence ;
key            = IDENT ;            (* must not start with `:`, `-`, `#` *)
scalar         = plain-scalar | dq-scalar | block-scalar ;
plain-scalar   = CHAR+ ;            (* spacing rule applies *)
dq-scalar      = DQUOTE CHAR* DQUOTE ;
block-scalar   = HEADER NEWLINE INDENTED-CONTENT ;
HEADER         = ( "|" | ">" ) [ "-" | "+" ] ;   (* `>` does NOT fold *)

(* Block 2 — Document fence *)
block-2        = FENCE-OPEN CONTENT FENCE-CLOSE ;
FENCE-OPEN     = "```" path "." format NEWLINE ;
FENCE-CLOSE    = "```" NEWLINE ;

(* Terminals *)
INDENTED-CONTENT = { any-char } ;   (* verbatim; indented past its owner *)
COLON-SP       = ":" ( SP | NEWLINE ) ;
DASH-SP        = "-" ( SP | NEWLINE ) ;
IDENT          = CHAR+ ;
SP             = U+0020 ;
DQUOTE         = U+0022 ;
NEWLINE        = U+000A ;
```
