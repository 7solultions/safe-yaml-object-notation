//! Numeric error codes for [`crate::error::SyonError`].
//!
//! Every error carries a code so a caller can ask "is this specifically a
//! duplicate-key error?" without matching on message text, which is fragile
//! across wording changes and impossible to keep in sync across three
//! independently-written parsers (see
//! `design/architecture/0004-independent-go-implementation.syon`).
//!
//! Codes are three digits, banded by block:
//!
//! - `1-99` general -- whole-file, or outside the block model entirely
//!   (`11-19` are decode-time, after a successful parse)
//! - `101-199` Block 1 -- records, including `|` block scalars
//! - `201-299` Block 2 -- document fences (` ```path.format `)
//!
//! Within a band, related problems share their low two digits, so `112` and
//! `212` would both be "anchor used where forbidden" -- in a record and in a
//! fence body respectively.
//!
//! The banding follows `spec/02-grammar.md`'s two-block model. It matches the
//! phase1 analyzer's numbering as well: the two schemes disagreed while the
//! `[[[ ... ]]]` escape hatch existed, and ADR 0007 removed it, which left
//! nothing for them to disagree about (see
//! `design/architecture/0006-phase1-block-numbering.syon`).
//!
//! Not every code is reachable in every implementation -- Rust carries plain
//! `[` and `{` through as scalar text where Go rejects them, and Go has no
//! configurable indentation step. `spec/05-error-codes.md` holds the full
//! table with per-implementation reachability.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ErrorCode {
    // ---- General / pre-block (1-99) ----
    /// Reserved: never constructed here. Invalid UTF-8 is caught by whoever
    /// reads the file (e.g. `fs::read_to_string`) before `parse` is called.
    InvalidUtf8 = 1,
    /// A tab character appears in a line's indentation prefix.
    TabInIndentation = 2,
    /// A line was not claimed by any construct -- neither a mapping entry, a
    /// sequence item, nor the body of a `|` block.
    UnexpectedContent = 3,
    /// Indentation is not a multiple of the configured step. Rust only; Go's
    /// indentation step is not configurable.
    IndentNotMultipleOfStep = 4,
    /// A scalar could not be decoded into the requested target type. Go only:
    /// Rust has no `Unmarshal` equivalent, returning a [`crate::ast::Value`]
    /// for the caller to interpret.
    DecodeTypeMismatch = 11,
    /// A node's shape did not match the requested target type (a mapping
    /// wanted where a sequence stood, or vice versa). Go only, as
    /// [`Self::DecodeTypeMismatch`].
    DecodeShapeMismatch = 12,
    /// Catch-all for a grammar failure with no more specific code. Rust only:
    /// this is what a raw pest parse error becomes.
    MalformedStructure = 90,

    // ---- Block 1: records (101-199) ----
    /// A mapping key starts with `:`, `-`, or `#`.
    KeyStartsWithOperator = 101,
    /// A mapping key is empty. Go only.
    EmptyKey = 102,
    /// The same key appears twice in one mapping.
    DuplicateKey = 103,

    /// `!x` / `!!x` explicit tag.
    ExplicitTag = 111,
    /// `&name` anchor.
    Anchor = 112,
    /// `*name` alias.
    Alias = 113,
    /// `{...}` flow mapping. Go only: Rust deliberately carries `{` through as
    /// scalar text, since it is an indicator only at node start and a
    /// pre-parse scan cannot tell the difference.
    FlowMapping = 114,
    /// `[...]` flow sequence. Go only, for the same reason as [`Self::FlowMapping`].
    FlowSequence = 115,
    /// `?` complex key.
    ComplexKey = 116,
    /// `---` explicit document-start marker after content has begun.
    DocumentStartMarker = 117,
    /// `...` explicit document-end marker. Rust only.
    DocumentEndMarker = 118,
    /// `[[[` or `]]]`, the literal escape hatch removed by ADR 0007. Rust
    /// only, and rejected by name so the message can say to use `|` instead.
    LiteralBlockRemoved = 119,

    /// A quoted scalar has no closing quote. Go only; in Rust this fails in
    /// the grammar and surfaces as [`Self::MalformedStructure`].
    UnterminatedQuotedString = 121,

    /// A sequence item writes `- key: |` without `allow_key_in_line_after_list`.
    /// Rust only; the option is Rust-side.
    CompactBlockScalarNeedsOption = 131,
    /// A sequence item mixes `key: value` with an indented non-mapping block.
    /// Rust only.
    SequenceItemMixesMappingAndBlock = 132,
    /// A sequence item has both inline text and an indented block. Rust only.
    SequenceItemInlineTextAndBlock = 133,

    // ---- Block 2: document fences (201-299) ----
    /// A fence info string is missing its `path.format` separator. Go only;
    /// in Rust the fence simply does not match `fence_open` and the line
    /// surfaces as [`Self::UnexpectedContent`].
    FenceInfoStringMalformed = 201,
    /// No matching closing ` ``` ` before end of input.
    UnterminatedFence = 202,
}

impl ErrorCode {
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SYON-{:03}", self.as_u16())
    }
}
