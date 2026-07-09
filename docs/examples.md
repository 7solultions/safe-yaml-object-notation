# Glossary example

The [`examples/glossary`](https://github.com/object-notation-environment/safe-yaml-object-notation/tree/main/examples/glossary)
directory is a worked example of using SYON as a self-describing data format:
a small schema, expressed in SYON itself, describing glossary entries that
are also SYON documents.

## The schema

[`examples/glossary/schema.syon`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/examples/glossary/schema.syon)
defines the field structure and cross-field constraints for a glossary entry,
using nothing but SYON's core features — mappings, sequences, and literal
blocks:

```syon
fields:
  term:
    type: string
    required: optional
    one-of-group: identity
    description: The full human-readable name of the concept.

  abbreviation:
    type: string
    required: optional
    one-of-group: identity
    description: Short-form abbreviation or acronym for the concept.
```

Cross-field constraints (like "at least one of `term` or `abbreviation`") are
described in a `constraints` section rather than enforced by the grammar
itself — SYON only defines the shape of the document, not schema validation.
See [the glossary schema convention](spec/04-glossary-schema.md) for the full
set of field groups and constraint rules.

## An entry

[`examples/glossary/entries/syon.syon`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/examples/glossary/entries/syon.syon)
is a conforming entry — it describes SYON itself:

```syon
abbreviation: SYON
term: Safe YAML Object Notation
id: syon-001
version: 0.9.0
description: [[[
  A human-writable data serialization format that is safe (no implicit
  typing, no executable constructs), simple (a small fixed set of markers),
  and structured (keys, lists, nesting). A member of the ONE family.
]]]
contexts:
  - data-formats
  - serialization
  - one-family
opposites:
  full-yaml: unrestricted YAML with tags, anchors, and implicit typing
relationships:
  see-also: yaml-001
  member-of: one-family
  inspired-by:
    - strictyaml-001
    - nestedtext-001
history:
  2026-06-27: v0.9.0 draft published
```

Notice how it uses all three SYON block types in one document: plain
mappings and sequences, a nested mapping (`relationships`), and a
[literal block](language.md#block-3-literal-escape-hatch) for the
multi-paragraph `description`.

## Try it

```bash
cargo run -p syon-cli -- examples/glossary/entries/syon.syon
```

prints the entry as JSON — see the [CLI reference](cli.md) for details.
