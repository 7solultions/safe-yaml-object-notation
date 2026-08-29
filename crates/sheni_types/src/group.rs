//! The four groups a Sheni type can belong to.
//!
//! The grouping is a property of the type system rather than a heading in a
//! document: every type descriptor reports its group, and the error-code
//! space is banded by it (see [`crate::error_code::SheniCode`]).
//!
//! There are four, and they classify a type by what it *is*. Where a
//! definition came from is a different axis and is not modelled here: a
//! struct an application declares in a schema is still a struct, and it
//! belongs to the layer above (see
//! `design/architecture/ADR_sheni_05__type_layer_boundary.syon`).
//!
//! See `design/architecture/ADR_sheni_01__primitives.syon`.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The group a type belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeGroup {
    /// Values that carry no interpretation beyond their own shape: booleans,
    /// numbers, characters, text. See [`crate::PrimitiveType`].
    Primitive,
    /// An interpretation laid over a primitive carrier -- a date, an email
    /// address, a currency code. The carrier is a primitive; the constraint
    /// is what makes it simple rather than primitive.
    Simple,
    /// A curated library of composite types shipped in the crate -- an
    /// address, a month, an amount of money. Enums and structs, closed like
    /// the two groups below it, so that two applications using Sheni mean the
    /// same thing by an address without agreeing on one first.
    Complex,
    /// A container over other types: a list keyed by position, a map keyed by
    /// a scalar. Constructed structurally from the types it holds, so it is
    /// closed in its constructors and open in what they construct.
    Collection,
}

impl TypeGroup {
    /// Every group, in the order the groups are numbered.
    pub const ALL: [TypeGroup; 4] = [
        TypeGroup::Primitive,
        TypeGroup::Simple,
        TypeGroup::Complex,
        TypeGroup::Collection,
    ];

    /// The group's name as it is written in a schema, lowercase and singular.
    pub const fn name(self) -> &'static str {
        match self {
            TypeGroup::Primitive => "primitive",
            TypeGroup::Simple => "simple",
            TypeGroup::Complex => "complex",
            TypeGroup::Collection => "collection",
        }
    }

    /// The reverse of [`Self::name`]. Case-sensitive: group names are written
    /// lowercase, and accepting other spellings would make two schemas that
    /// disagree on capitalisation both valid.
    pub fn from_name(name: &str) -> Option<Self> {
        TypeGroup::ALL.into_iter().find(|g| g.name() == name)
    }

    /// The first code in this group's error band -- 100 for primitives, 200
    /// for simple, and so on. General errors sit below 100, outside any band.
    pub const fn code_band(self) -> u16 {
        match self {
            TypeGroup::Primitive => 100,
            TypeGroup::Simple => 200,
            TypeGroup::Complex => 300,
            TypeGroup::Collection => 400,
        }
    }
}

impl fmt::Display for TypeGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for group in TypeGroup::ALL {
            assert_eq!(TypeGroup::from_name(group.name()), Some(group));
        }
    }

    #[test]
    fn unknown_and_miscased_names_are_rejected() {
        assert_eq!(TypeGroup::from_name("Primitive"), None);
        assert_eq!(TypeGroup::from_name("primitives"), None);
        assert_eq!(TypeGroup::from_name(""), None);
    }

    #[test]
    fn bands_are_distinct_and_ordered() {
        let bands: Vec<u16> = TypeGroup::ALL.iter().map(|g| g.code_band()).collect();
        assert_eq!(bands, vec![100, 200, 300, 400]);
    }
}
