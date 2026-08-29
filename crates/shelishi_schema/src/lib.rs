//! Shelishi -- the schema layer over Sheni.
//!
//! Sheni's four groups are all closed. Its curated complex library holds what
//! is common to applications that have nothing else in common, which by
//! construction leaves out everything an application actually is -- its
//! orders, its invoices, its device readings.
//!
//! This layer is where those are declared: enums and structs an application
//! writes as a SYON schema, read at runtime. They are the *same* enum and the
//! same struct [`sheni_types`] defines. Only the source of the definition
//! differs, which is why this is a layer rather than a fifth group -- see
//! `design/architecture/ADR_sheni_05__type_layer_boundary.syon`.
//!
//! The dependency runs one way and cannot be made to run the other: a curated
//! shape can never name a user-declared one.
//!
//! Errors take the 501-599 band, reserved for this crate by the layer below.
//!
//! Not implemented. The design is recorded in
//! `design/architecture/ADR_shelishi_01__schema_based_types.syon`.

// TODO: implement shelishi_schema, per ADR shelishi_01.
