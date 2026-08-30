# PYON — notes

Status: **concept**. Nothing is built. The syntax below is as specified by
Felix on 2026-08-29; the analysis after it is mine and is not decided.

## The idea

PYON is SYON's model expressed in **Python literal syntax**, so that a document
is a module:

```python
import config          # resolves config.pyon on sys.path

config.database["host"]
```

That inverts SYON's relationship to its host language rather than repeating it.
SYON is a safe subset of YAML; PYON is a safe subset of **Python**. Importability
is not bolted on by an import hook over an alien format — it falls out of the
file already being Python.

It covers **Blocks 1 to 3** at level 1 parsing, human-friendly but Pythonic.

## Syntax

### Block 1 — core elements

- **Comments** are exactly as in SYON.
- **Mappings** are `{key: value}`.
- **Sequences** are `[value]`.
- **Strings** are quoted, `"..."` or `'...'`.

### Block 2 — multiple files / objects

SYON separates embedded documents with a fence carrying a
`{path}/{file_name}.{extension}` info string. PYON has no such separator.
Instead each embedded document is a **binding**:

```python
{file_name}__{extension} = ...
```

The double underscore stands in for the dot, so the module namespace itself is
the multi-document container.

### Block 3 — multiline text

Instead of SYON's `|` block scalar, a Python multiline string.

## Worked example

SYON, and what `syon.parse` returns today:

```
name: Alice
age: 30
contexts:
  - data-formats
  - serialization
```

```python
{'name': 'Alice', 'age': '30', 'contexts': ['data-formats', 'serialization']}
```

The same thing in PYON:

```python
# a person
person = {
    "name": "Alice",
    "age": 30,
    "contexts": ["data-formats", "serialization"],
}
```

Note `age`: `30` in PYON is an `int`, where SYON gives `'30'`. See below.

## What this settles

The earlier open question — whether PYON is a format or an import mechanism —
is answered: **it is a format**, and being a format that is also valid Python is
what makes it importable. Both readings were the same thing seen from two ends.

It also gives a clean safety story, which SYON needs an explicit
forbidden-construct scan to achieve. A PYON loader never executes the file: it
runs `ast.parse`, walks the tree, and accepts only assignments whose right-hand
side is a literal. Anything else — a call, an import, an f-string, a dunder — is
a parse error, not a security control bolted on afterwards.

And Block 4 turns out not to be needed for any of this.

## Tensions to resolve

Four places where "human friendly", "Pythonic", and "directly importable" pull
against each other. The first is the sharp one.

**1. `{key: value}` is not valid Python with a bare key.** In a Python dict
literal a bare `key` is a *variable reference*. `{name: "Alice"}` parses fine —
`ast.parse` returns a `Dict` — but evaluating it raises `NameError`, and
`ast.literal_eval` rejects it with `ValueError: malformed node or string ...
Name(id='name')`. Keys must be quoted — `{"name": "Alice"}` —
for the file to be importable by ordinary CPython. So either:

- *Strict subset.* Keys are quoted. Any Python can read a `.pyon` file with
  `ast.literal_eval`, no hook needed, and "directly importable" is literally
  true. Cost: `{"name": "Alice"}` is noisier than `name: Alice`, and the
  quoting is the main thing people dislike about JSON as a config format.
- *PYON dialect.* Bare keys allowed, read as strings. Much friendlier, and
  closer to SYON. Cost: the file is no longer valid Python, only
  Python-*shaped*; it can only be read through PYON's own loader, and every
  editor, linter and type checker that assumes `.py` semantics will be wrong
  about it.

This is the decision the rest hangs on. The example above assumes the strict
subset only because that is the reading under which the code shown actually
runs.

**2. The strings-only boundary is gone.** `spec/02-grammar.md` guarantees "all
content is a string at the parse boundary — no implicit type coercion", and
`ADR_sheni_*` builds a whole type layer on top of that. Python literal syntax
carries types intrinsically: `30` is an `int`, `true`/`false` are `True`/`False`,
`None` exists. That is almost certainly the right call for a Python-facing
format — but it means PYON and SYON are **not round-trippable** without a
schema, and `sheni_types`/`shelishi_schema` have much less to do on the PYON
side. Worth stating as a deliberate divergence rather than discovering it later.

**3. Block 2 loses the path.** SYON's fence info string carries
`{path}/{file_name}.{extension}` and exposes `path` and `format` separately. A
Python identifier cannot contain `/`, so `path/to/resource.json` has nowhere to
put `path/to/`. Options: drop paths, flatten the separator into the name, or
keep a companion mapping. Related: `__` is ambiguous when the filename itself
contains one — `foo__bar.json` becomes `foo__bar__json`, which does not split
back cleanly — and a name beginning with `_` is skipped by `from x import *`.

**4. Multiline strings do not dedent or chomp.** SYON's block scalar dedents the
body by its common leading indentation and offers `|`, `|-`, `|+` chomping.
Python's triple-quoted string does neither: indentation written for readability
ends up *in the value*, and the trailing newline is whatever you typed. Either
PYON applies `textwrap.dedent`-style handling at parse time (friendly, but no
longer exactly what Python evaluates — which reopens tension 1) or authors
must write flush-left multiline strings inside indented structures.

## Open questions

- **"Level 1 parsing"** — the term is from the spec above and I do not have a
  definition for it. Conformance level? First nesting level only? It bounds the
  scope of the whole thing, so it should be pinned down first.
- **Comments.** "Exactly the same as SYON" — SYON's lexer uses `# ` with a
  required space, Python accepts a bare `#`. Which rule wins? Requiring the
  space makes valid Python invalid PYON.
- **Duplicate keys.** SYON rejects them (`DUPLICATE_KEY`, code 103). Python dict
  literals silently keep the last. `ast` exposes the duplicate before
  evaluation, so PYON can still reject it — but it has to choose to.
- **The top-level shape.** Is a PYON file's namespace itself the document's root
  mapping, with Block 2's `name__ext` bindings simply being top-level keys under
  a naming convention? That reading is tidy and seems implied; worth confirming.
- **Which implementations.** `ADR_syon_04` commits SYON to independent Rust and
  Go parsers. Does PYON inherit that, or is a Python-only implementation over
  `ast` acceptable given the format is defined by Python's own grammar?
- **Errors.** Reuse `syon.ErrorCode` and `SyonError`, or a PYON code range?
  Does an import failure surface as `ImportError` too?
- **Caching and reload.** `.pyc`-equivalent caching, or reparse per import?
- **Static tooling.** Under the strict subset a type checker sees a `.pyon` file
  as nothing at all. Emit `.pyi` stubs?
- **Packaging.** Separate distribution, or an extra on the `syon` wheel?
- **Davar.** Whether PYON ships as a core-side provider class there. The Davar
  ADR defers this to "block four semantics", which on this design is moot — the
  blocker should be restated.

## Next step

Settle tension 1 — quoted keys or bare keys. Strict subset buys real
importability and zero tooling work; the dialect buys the friendliness that
distinguishes PYON from just writing a `.py` file. Nearly every other question
above reads differently depending on which one it is.
