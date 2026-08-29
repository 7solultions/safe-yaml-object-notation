# Shelishi (3.Layer): Schema

Sheni's four type groups are closed. Its curated complex library holds the
composite types common to applications that have nothing else in common —
which, by construction, leaves out everything an application actually is.

Shelishi is where those get declared: enums and structs written as a SYON
schema and read at runtime.

## Status

Not implemented. The design is recorded in
[`ADR shelishi_01`](../../design/architecture/ADR_shelishi_01__schema_based_types.syon),
and the reason this is a layer rather than a fifth Sheni group in
[`ADR sheni_05`](../../design/architecture/ADR_sheni_05__type_layer_boundary.syon).

## What it borrows

A declared enum and a declared struct are the *same* shapes Sheni's complex
group defines — same variants over a backing primitive, same named and typed
fields, same rules on required fields, unrecognised keys, and declaration
order. Only where the definition comes from differs. If the two ever disagree
about what a struct is, Sheni is right and this crate is broken.

A declared type may name a built-in from any of Sheni's four groups, so a
schema starts from the curated vocabulary rather than from primitives:

```syon
types:
  Order:
    kind: struct
    fields:
      placed_at:
        type: timestamp
      ship_to:
        type: address
      lines:
        type: list<Order_line>
      note:
        type: string
        optional: yes
```

The dependency runs one way and cannot be made to run the other: a curated
shape can never name a user-declared one, because Sheni's complex group is
closed.
