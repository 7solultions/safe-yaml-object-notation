//! `examples/planner/` read through Sheni, so the example cannot drift from
//! the crate that gives it meaning.
//!
//! The example exists to show what the soft types are for, and the claim it
//! makes is a strong one: an absent optional field reads as "nobody knows"
//! rather than as a zero, a `false`, or an epoch date. That claim is checked
//! here rather than asserted in prose.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use sheni_types::{PrimitiveType, SimpleType, SoftPrimitiveType, Value};

/// The task schema, as `examples/planner/schema.syon` declares it: field
/// name, type name, and whether the field may be left out.
const FIELDS: [(&str, &str, bool); 8] = [
    ("id", "uuid", false),
    ("title", "string", false),
    ("due", "soft_date", true),
    ("window", "soft_date_range", true),
    ("estimate", "duration_iso", false),
    ("priority", "u8", false),
    ("assignee_count", "soft_u8", true),
    ("blocked", "soft_bool", true),
];

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/planner")
}

fn task(name: &str) -> BTreeMap<String, Value> {
    let path = examples_dir().join("tasks").join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let file = syon_parser::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    match &file.documents[0].body {
        Value::Mapping(entries) => entries
            .iter()
            .map(|e| (e.key.clone(), e.value.clone()))
            .collect(),
        other => panic!("{}: expected a mapping, found {other:?}", path.display()),
    }
}

/// Read one field at the type the schema declares for it, or take the type's
/// fallback when the key is absent. This is the whole of what layer 3 will do
/// for a scalar field, written out so the example demonstrates it.
fn read(fields: &BTreeMap<String, Value>, name: &str, type_name: &str) -> String {
    let node = fields.get(name);
    match (node, type_name) {
        (Some(v), t) if PrimitiveType::from_name(t).is_some() => PrimitiveType::from_name(t)
            .unwrap()
            .read_value(v)
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .to_string(),
        (Some(v), t) if SoftPrimitiveType::from_name(t).is_some() => {
            SoftPrimitiveType::from_name(t)
                .unwrap()
                .read_value(v)
                .unwrap_or_else(|e| panic!("{name}: {e}"))
                .to_string()
        }
        (Some(v), t) => SimpleType::from_name(t)
            .unwrap_or_else(|| panic!("{name}: no type named {t}"))
            .read_value(v)
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .to_string(),
        (None, t) => {
            if let Some(soft) = SoftPrimitiveType::from_name(t) {
                soft.fallback().to_string()
            } else if let Some(simple) = SimpleType::from_name(t) {
                simple
                    .fallback()
                    .unwrap_or_else(|| panic!("{name}: {t} has no fallback"))
                    .to_string()
            } else {
                panic!("{name}: {t} has no fallback, so the field cannot be optional")
            }
        }
    }
}

/// Every type the schema names is a type this crate ships, and every field
/// declared optional has one with a fallback. ADR sheni_03 makes the second
/// half of that the condition for optionality at all.
#[test]
fn the_schema_names_only_types_that_exist_and_optional_only_where_a_fallback_does() {
    for (field, type_name, optional) in FIELDS {
        let has_fallback = SoftPrimitiveType::from_name(type_name).is_some()
            || SimpleType::from_name(type_name).is_some_and(|t| t.fallback().is_some());
        let exists = has_fallback
            || PrimitiveType::from_name(type_name).is_some()
            || SimpleType::from_name(type_name).is_some();

        assert!(exists, "{field}: no type named {type_name}");
        assert_eq!(
            has_fallback, optional,
            "{field} declared optional: {optional}"
        );
    }
}

#[test]
fn a_mostly_known_task_reads_at_every_declared_type() {
    let t = task("roadmap.syon");
    assert_eq!(read(&t, "title", "string"), "Draft the product roadmap");
    // A quarter: sub-year grouping code 35 is Q3, and it survives the read.
    assert_eq!(read(&t, "due", "soft_date"), "2026-35");
    assert_eq!(read(&t, "window", "soft_date_range"), "2026-07/2026-09");
    assert_eq!(read(&t, "assignee_count", "soft_u8"), "3");
    assert_eq!(read(&t, "blocked", "soft_bool"), "false");
}

/// Written-out unknowns. The document says "nobody knows" rather than staying
/// silent, and every one of them is a value rather than a gap.
#[test]
fn a_task_full_of_unknowns_reads_them_as_values() {
    let t = task("hiring.syon");
    assert_eq!(read(&t, "due", "soft_date"), "XXXX");
    assert_eq!(read(&t, "assignee_count", "soft_u8"), "unknown");
    assert_eq!(read(&t, "blocked", "soft_bool"), "unknown");
    // `..` is an open end: the window genuinely has no close, as distinct
    // from having one nobody recorded.
    assert_eq!(read(&t, "window", "soft_date_range"), "2026-10/..");
}

/// The point of the whole family: an absent optional field falls back to an
/// unknown, never to a zero, a `false`, or an epoch.
#[test]
fn absent_optional_fields_fall_back_to_unknowns_and_not_to_zeroes() {
    let t = task("audit.syon");
    for (field, type_name, optional) in FIELDS {
        if !optional {
            continue;
        }
        assert!(!t.contains_key(field), "{field} should be absent here");
        let value = read(&t, field, type_name);
        assert!(
            value == "unknown" || value == "XXXX" || value == "XXXX/XXXX",
            "{field} fell back to {value:?}"
        );
    }
    assert_ne!(read(&t, "assignee_count", "soft_u8"), "0");
    assert_ne!(read(&t, "blocked", "soft_bool"), "false");
}

/// The required fields are required because their types have no fallback,
/// which is ADR sheni_03's rule and not a preference of the schema author.
#[test]
fn the_required_fields_are_required_because_their_types_have_no_fallback() {
    for (field, type_name, optional) in FIELDS {
        if optional {
            continue;
        }
        let fallback = SimpleType::from_name(type_name).and_then(|t| t.fallback());
        assert!(fallback.is_none(), "{field}: {type_name} has a fallback");
        assert!(
            SoftPrimitiveType::from_name(type_name).is_none(),
            "{field}: {type_name} is soft"
        );
    }
}
