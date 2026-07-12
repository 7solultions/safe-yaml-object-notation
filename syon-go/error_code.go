package syon

import "fmt"

// ErrorCode is a numeric error code for Error, mirroring
// crates/syon-parser/src/error_code.rs.
//
// Ranges: 1-99 Phase 1 (general, pre-block); 101-199 Block 1 (records);
// 201-299 Block 2 (literal blocks, [[[ ... ]]]); 301-399 Block 3 (document
// fences, ```path.format```).
//
// Terminology note: Block 2/3 here use the phase1 tool's numbering (see
// docs/decisions/0006-phase1-block-numbering.syon), the OPPOSITE of
// spec/02-grammar.md's (which has these two swapped) -- deliberately, per
// the same ADR's discipline. See spec/05-error-codes.md for the full
// table, including per-implementation reachability notes.
type ErrorCode int

const (
	// Phase 1: general, pre-block (1-99)

	// CodeInvalidUTF8 is reserved: not currently constructed by this
	// package. Invalid UTF-8 would be caught by the caller reading the
	// file before Parse/Unmarshal is ever invoked.
	CodeInvalidUTF8               ErrorCode = 1
	CodeTabInIndentation          ErrorCode = 2
	CodeUnexpectedTrailingContent ErrorCode = 3
	// CodeMalformedStructure is a catch-all for a structural failure not
	// covered by a more specific code above.
	CodeMalformedStructure ErrorCode = 90

	// Block 1: records (101-199)

	CodeKeyStartsWithOperator ErrorCode = 101
	CodeEmptyKey              ErrorCode = 102
	CodeDuplicateKey          ErrorCode = 103
	CodeExplicitTag           ErrorCode = 111
	CodeAnchor                ErrorCode = 112
	CodeAlias                 ErrorCode = 113
	CodeFlowMapping           ErrorCode = 114
	CodeFlowSequence          ErrorCode = 115
	CodeComplexKey            ErrorCode = 116
	CodeDocumentStartMarker   ErrorCode = 117
	// CodeDocumentEndMarker is reserved: this package only rejects `---`,
	// not `...` (see spec/05-error-codes.md).
	CodeDocumentEndMarker        ErrorCode = 118
	CodeUnterminatedQuotedString ErrorCode = 121

	// Block 2: literal blocks, [[[ ... ]]] (201-299)

	CodeUnterminatedLiteralBlock ErrorCode = 202
	// Codes 211-218 (forbidden construct inside a literal block) are
	// reserved and not reachable in this package: literal block bodies
	// are opaque, verbatim text, never scanned for forbidden constructs.

	// Block 3: document fences, ```path.format``` (301-399)

	CodeFenceInfoStringMalformed ErrorCode = 301
	CodeUnterminatedFence        ErrorCode = 302
	// Codes 311-318 (forbidden construct inside fence content) are
	// reserved and not reachable in this package either: fence bodies are
	// opaque, verbatim text here (unlike the Rust implementation -- see
	// docs/decisions/0006-phase1-block-numbering.syon).
)

func (c ErrorCode) String() string { return fmt.Sprintf("SYON-%03d", int(c)) }
