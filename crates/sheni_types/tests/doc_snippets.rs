//! The snippets in `crates/sheni_types/README.md` and `docs/types.md`,
//! compiled. Documentation that drifts from the crate is worse than none.

use sheni_types::{PrimitiveType, SheniCode, SoftPrimitiveType};

#[test]
fn soft_primitive_snippet() {
    let count = SoftPrimitiveType::from_name("soft_u32").unwrap();
    assert_eq!(count.fallback().to_string(), "unknown");
    assert_ne!(count.fallback(), count.read("0").unwrap());

    assert_eq!(
        count.read("007").unwrap_err().code(),
        SheniCode::LeadingZero
    );
}

#[test]
fn soft_bool_snippet() {
    let t = SoftPrimitiveType::from_name("soft_bool").unwrap();
    let (no, dunno) = (t.read("false").unwrap(), t.read("unknown").unwrap());
    assert_eq!(no.and(&dunno).unwrap(), no);
    assert_eq!(dunno.not().unwrap(), dunno);
}

#[test]
fn there_is_no_soft_string_snippet() {
    assert_eq!(SoftPrimitiveType::new(PrimitiveType::String), None);
}

// ---- docs/types.md ----

#[test]
fn reading_is_checked_snippet() {
    use sheni_types::PrimitiveValue;

    let u8_type = PrimitiveType::from_name("u8").unwrap();
    assert_eq!(
        u8_type.read("300").unwrap_err().code(),
        SheniCode::IntegerOutOfRange
    );
    assert_eq!(
        PrimitiveType::Boolean.read("no"),
        Ok(PrimitiveValue::Boolean(false))
    );
    assert_eq!(
        PrimitiveType::String.read("no"),
        Ok(PrimitiveValue::String("no".to_string()))
    );
}

#[test]
fn simple_normalises_snippet() {
    use sheni_types::SimpleType;

    assert_eq!(
        SimpleType::IpAddress
            .read("2001:0DB8::1")
            .unwrap()
            .to_string(),
        "2001:db8::1"
    );
}

#[test]
fn soft_date_snippet() {
    use sheni_types::{Precision, SimpleType};

    let due = SimpleType::SoftDate.read("2026-35").unwrap();
    assert_eq!(due.precision(), Some(Precision::Season));

    assert!(SimpleType::Date.read("2026-08").is_err());
    assert!(SimpleType::SoftDate.read("2026-08").is_ok());
}

#[test]
fn error_code_snippet() {
    let err = PrimitiveType::Boolean.read("unknown").unwrap_err();
    assert_eq!(err.code(), SheniCode::UnknownAtStrictType);
    assert_eq!(
        err.message(),
        "`unknown` is a value of `soft_bool`, not of `bool`"
    );
}

/// The tables in `docs/types.md` claim these forms mean what they say.
#[test]
fn the_soft_date_tables_are_accurate() {
    use sheni_types::{IntervalEndpoint, Precision, SimpleType};

    for (literal, precision) in [
        ("2026-08-12", Precision::Day),
        ("2026-08", Precision::Month),
        ("2026-35", Precision::Season),
        ("2026", Precision::Year),
        ("2026-08-12?", Precision::Day),
        ("2026~", Precision::Year),
        ("XXXX", Precision::Year),
    ] {
        let v = SimpleType::SoftDate.read(literal).unwrap();
        assert_eq!(v.precision(), Some(precision), "{literal}");
    }
    assert_eq!(SimpleType::SoftDate.fallback().unwrap().to_string(), "XXXX");

    for literal in ["2026-07/2026-09", "2026-10/..", "2026-10/", "../2026-10"] {
        let v = SimpleType::SoftDateRange.read(literal).unwrap();
        assert_eq!(v.to_string(), literal, "{literal}");
    }
    assert!(matches!(
        SimpleType::SoftDateRange
            .read("2026-10/..")
            .unwrap()
            .interval()
            .unwrap()
            .end,
        IntervalEndpoint::Open
    ));
    assert!(matches!(
        SimpleType::SoftDateRange
            .read("2026-10/")
            .unwrap()
            .interval()
            .unwrap()
            .end,
        IntervalEndpoint::Unknown
    ));
    assert_eq!(
        SimpleType::SoftDateRange.fallback().unwrap().to_string(),
        "XXXX/XXXX"
    );
}
