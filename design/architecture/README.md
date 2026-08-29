# Architecture Decision Records

This directory records the architecturally significant decisions made on
this project, in a lightweight [Michael Nygard ADR
format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
adapted to fields, expressed as SYON documents — each ADR is itself a piece
of real SYON content, validated by both the Rust and Go implementations in
CI (see `examples-valid` and `go-build` in
[`.github/workflows/ci.yml`](https://github.com/object-notation-environment/safe-yaml-object-notation/blob/main/.github/workflows/ci.yml)).

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [syon_01](ADR_syon_01__record_architecture_decission.syon) | Record architecture decisions | Accepted |
| [syon_02](ADR_syon_02__pest_as_the_rust_parsing_engine.syon) | Use pest as the Rust parsing engine | Accepted |
| [syon_03](ADR_syon_03__preflight_scan_for_forbidden_constructs.syon) | Preflight text scan for forbidden constructs | Accepted |
| [syon_04](ADR_syon_04__independent_go_implementation.syon) | Independent Go implementation instead of FFI bindings | Accepted |
| [syon_05](ADR_syon_05__block_1_only_yaml_compatibility.syon) | Only Block 1 is YAML-compatible | Accepted |
| [syon_06](ADR_syon_06__phase1_block_numbering.syon) | Phase1 report uses its own block numbering, distinct from the grammar spec | Superseded by syon_07 |
| [syon_07](ADR_syon_07__remove_the_literal_escape_hatch.syon) | Remove the literal escape hatch in favour of YAML block scalars | Accepted |
| [syon_08](ADR_syon_08__parse_error_codes.syon) | Numeric error codes for parse errors, mirrored per language | Accepted |
| [syon_09](ADR_syon_09__split_and_compact_multi_document.syon) | Split and compact subcommands for multi-document files | Proposed |
| [hodesh_01](ADR_hodesh_01__two_crates_and_a_day_count.syon) | Calendars convert through one day count, and the count lives in a crate below them | Proposed |
| [hodesh_02](ADR_hodesh_02__month_is_the_mean_lunation.syon) | The month is the mean lunation, and the 29/30 alternation is its consequence rather than its rule | Proposed |
| [hodesh_03](ADR_hodesh_03__metonic_year_and_appended_leap_month.syon) | The year is Metonic, and the leap month is appended as month 13 rather than inserted | Proposed |
| [hodesh_04](ADR_hodesh_04__numbered_months_and_a_new_moon_epoch.syon) | Months are numbered rather than named, and year zero begins at the first new moon of 2000 | Proposed |
| [shelishi_01](ADR_shelishi_01__schema_based_types.syon) | The schema layer supplies types at runtime, reusing the shapes sheni already defines | Proposed |
| [sheni_01](ADR_sheni_01__primitives.syon) | Primitive type set, spelling, and literal forms for Sheni | Accepted |
| [sheni_02](ADR_sheni_02__simple_types.syon) | Simple types delegate to the standard crates, and reading them normalises | Accepted |
| [sheni_03](ADR_sheni_03__complex_types.syon) | Complex types are a curated library shipped in code, not a way to declare types | Proposed |
| [sheni_04](ADR_sheni_04__collections.syon) | Collections are parameterised by their element types, and a map key is read like any other value | Proposed |
| [sheni_05](ADR_sheni_05__type_layer_boundary.syon) | The type layer stops at four kinds, and user-defined types belong to the layer above | Accepted |
| [sheni_06](ADR_sheni_06__soft_dates.syon) | A date that is not fully known is a type, not a date field that might be missing | Accepted |
| [sheni_07](ADR_sheni_07__soft_bool.syon) | A boolean that may be unknown is a three-valued enum, and `bool` stays two-valued | Accepted |
| [sheni_08](ADR_sheni_08__soft_types.syon) | An unknown belongs in the value space, and the `soft_` prefix marks the types that hold one | Accepted |
| [sheni_09](ADR_sheni_09__soft_primitives.syon) | Every primitive gets a soft twin except `string`, which cannot spell its own unknown | Accepted |
| [sheni_10](ADR_sheni_10__soft_date_ranges.syon) | A range whose ends may be unknown is one type, and the standard chooses its fallback | Accepted |
| [shlita_01](ADR_shlita_01__two_crates_and_scope.syon) | IEC 61131-3 splits into a type vocabulary and a scan runtime, and neither one is a language | Proposed |
| [shlita_02](ADR_shlita_02__fbd_and_sfc_are_documents.syon) | FBD and SFC are documents rather than syntaxes, and PLCopen already names their parts | Proposed |
| [shlita_03](ADR_shlita_03__structured_text_dialect.syon) | Structured Text takes Python's statements and IEC's types, and is not Python | Proposed |

Each layer numbers its own decisions, and both the file name and the
`identifier` field carry the prefix that says which layer — `syon` for the
parser and the language, `sheni` for the type layer above it, `shelishi` for
the schema layer above that, `hodesh` for the calendar crates and `shlita` for the
IEC 61131-3 control crates, both of which sit alongside rather than on top of
the SYON stack.

## Record schema

Each ADR is a `.syon` file with one top-level `architecture-decision-record`
mapping:

```syon
architecture-decision-record:
  identifier: "0009"
  title: Example decision title
  status: accepted
  date: "2026-08-23"
  deciders:
    - felix
  superseded-by: ""
  open-questions:
    - Something this record does not settle, and knows it does not settle.
  context: |
    What's the issue we're seeing that motivates this decision?
  decision: |
    What are we going to do about it?
  consequences:
    positive:
      - A good outcome of this decision.
    negative:
      - A trade-off or cost accepted along with it.
  alternatives-rejected:
    -
      name: An option that was considered
      reason: Why it wasn't chosen.
```

`open-questions` comes before `context`, deliberately: a reader should meet
what the record leaves open before the argument for what it closes, so the
limits of the decision are not something they have to reach the end to find.
Omit the field entirely on a record that settles everything it touches.

Note the `-` starting an `alternatives-rejected` entry must be alone on its
own line, with the nested `name`/`reason` mapping indented underneath — SYON
does not currently support a mapping key starting on the same line as the
list marker (e.g. `- name: ...`).

## Adding a new ADR

Copy this schema into a new `ADR_<layer>_<NN>__<short_title>.syon` file,
numbered sequentially within its layer, and add a row to the index above. Never edit or delete an
accepted ADR to reflect a later change of mind — write a new one that
supersedes it (setting the new ADR's context to reference the old one, and
the old ADR's `superseded-by` field to the new one's identifier).
