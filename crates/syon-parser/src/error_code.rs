//! Numeric error codes for [`crate::error::SyonError`].
//!
//! Ranges: 1-99 Phase 1 (general, pre-block); 101-199 Block 1 (records);
//! 201-299 Block 2 (literal blocks, `[[[ ... ]]]`); 301-399 Block 3
//! (document fences, ` ``` `path.format` `` `).
//!
//! Terminology note: Block 2/3 here use the phase1 tool's numbering (see
//! `docs/decisions/0006-phase1-block-numbering.syon`), the OPPOSITE of
//! `spec/02-grammar.md`'s (which has these two swapped) -- deliberately,
//! per the same ADR's discipline. See `spec/05-error-codes.md` for the
//! full table, including per-implementation reachability notes.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ErrorCode {
    // ---- Phase 1: general, pre-block (1-99) ----
    /// Reserved: not currently constructed by SyonError. Invalid UTF-8 is
    /// caught by the caller reading the file (e.g. `fs::read_to_string`)
    /// before `parse`/`parse_document` is ever invoked.
    InvalidUtf8 = 1,
    /// A tab character was found in a line's indentation prefix.
    TabInIndentation = 2,
    /// The document was not fully consumed by any recognized construct.
    UnexpectedTrailingContent = 3,
    /// Catch-all for a grammar/structural failure not covered by a more
    /// specific code above.
    MalformedStructure = 90,

    // ---- Block 1: records (101-199) ----
    /// A mapping key starts with `:`, `-`, or `#`.
    KeyStartsWithOperator = 101,
    /// A mapping key is empty.
    EmptyKey = 102,
    /// The same key appears twice in one mapping.
    DuplicateKey = 103,
    /// `!x` / `!!x` explicit tag.
    ExplicitTag = 111,
    /// `&name` anchor.
    Anchor = 112,
    /// `*name` alias.
    Alias = 113,
    /// `{...}` flow mapping.
    FlowMapping = 114,
    /// `[...]` flow sequence.
    FlowSequence = 115,
    /// `?` complex key.
    ComplexKey = 116,
    /// `---` explicit document-start marker.
    DocumentStartMarker = 117,
    /// `...` explicit document-end marker.
    DocumentEndMarker = 118,
    /// A double-quoted string has no closing quote.
    UnterminatedQuotedString = 121,

    // ---- Block 2: literal blocks, `[[[ ... ]]]` (201-299) ----
    /// No matching `]]]` before EOF.
    UnterminatedLiteralBlock = 202,
    /// Reserved, not currently reachable in either implementation: literal
    /// block bodies are exempted from forbidden-construct scanning.
    LiteralExplicitTag = 211,
    LiteralAnchor = 212,
    LiteralAlias = 213,
    LiteralFlowMapping = 214,
    LiteralFlowSequence = 215,
    LiteralComplexKey = 216,
    LiteralDocumentStartMarker = 217,
    LiteralDocumentEndMarker = 218,

    // ---- Block 3: document fences, ` ``` `path.format` `` ` (301-399) ----
    /// The fence info string is missing its `path.format` separator.
    FenceInfoStringMalformed = 301,
    /// No matching closing ` ``` ` before EOF.
    UnterminatedFence = 302,
    /// Reachable today only in the Rust implementation -- see
    /// `docs/decisions/0006-phase1-block-numbering.syon`'s note on fence
    /// content not being exempted from forbidden-construct scanning there.
    /// Not reachable in Go, which treats fence content as opaque.
    FenceExplicitTag = 311,
    FenceAnchor = 312,
    FenceAlias = 313,
    FenceFlowMapping = 314,
    FenceFlowSequence = 315,
    FenceComplexKey = 316,
    FenceDocumentStartMarker = 317,
    FenceDocumentEndMarker = 318,
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
