package syon

import "fmt"

// ErrorCode is a numeric error code carried by every Error, mirroring
// crates/syon-parser/src/error_code.rs.
//
// Codes are three digits, banded by block:
//
//	1-99    general -- whole-file, or outside the block model entirely
//	        (11-19 are decode-time, after a successful parse)
//	101-199 Block 1 -- records, including | block scalars
//	201-299 Block 2 -- document fences (```path.format)
//
// The banding follows spec/02-grammar.md's two-block model, which the phase1
// analyzer now shares: the two numbering schemes disagreed only while the
// [[[ ... ]]] escape hatch existed, and ADR 0007 removed it (see
// design/architecture/0006-phase1-block-numbering.syon).
//
// Not every code is reachable here -- this package has no configurable
// indentation step, and it rejects flow collections that the Rust parser
// carries through as scalar text. spec/05-error-codes.md holds the full
// table with per-implementation reachability.
type ErrorCode int

const (
	// General / pre-block (1-99)

	// CodeInvalidUTF8 is reserved: never constructed here. Invalid UTF-8 is
	// caught by whoever reads the file, before Parse is called.
	CodeInvalidUTF8       ErrorCode = 1
	CodeTabInIndentation  ErrorCode = 2
	CodeUnexpectedContent ErrorCode = 3
	// CodeIndentNotMultipleOfStep is reserved: Rust only, where the
	// indentation step is configurable via ParseOptions.
	CodeIndentNotMultipleOfStep ErrorCode = 4
	// Decode-time (11-19), after a successful parse. Go only: Rust has no
	// Unmarshal equivalent, returning a Value for the caller to interpret.

	// CodeDecodeTypeMismatch is a scalar that could not be decoded into the
	// requested target type.
	CodeDecodeTypeMismatch ErrorCode = 11
	// CodeDecodeShapeMismatch is a node whose shape did not match the
	// requested target type -- a mapping wanted where a sequence stood.
	CodeDecodeShapeMismatch ErrorCode = 12

	// CodeMalformedStructure is reserved: Rust only, where it carries a raw
	// pest grammar failure. This package has no equivalent catch-all.
	CodeMalformedStructure ErrorCode = 90

	// Block 1: records (101-199)

	CodeKeyStartsWithOperator ErrorCode = 101
	CodeEmptyKey              ErrorCode = 102
	CodeDuplicateKey          ErrorCode = 103

	CodeExplicitTag  ErrorCode = 111
	CodeAnchor       ErrorCode = 112
	CodeAlias        ErrorCode = 113
	CodeFlowMapping  ErrorCode = 114
	CodeFlowSequence ErrorCode = 115
	CodeComplexKey   ErrorCode = 116

	CodeDocumentStartMarker ErrorCode = 117
	// CodeDocumentEndMarker is reserved: this package rejects `---` but not
	// `...`. Rust rejects both.
	CodeDocumentEndMarker   ErrorCode = 118
	CodeLiteralBlockRemoved ErrorCode = 119

	CodeUnterminatedQuotedString ErrorCode = 121

	// Codes 131-133 (sequence-item shape errors) are reserved: Rust only,
	// where they concern the allow_key_in_line_after_list option.

	// Block 2: document fences (201-299)

	CodeFenceInfoStringMalformed ErrorCode = 201
	CodeUnterminatedFence        ErrorCode = 202
)

func (c ErrorCode) String() string { return fmt.Sprintf("SYON-%03d", int(c)) }
