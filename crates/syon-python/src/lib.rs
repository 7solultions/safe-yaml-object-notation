use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use syon_parser::{ErrorCode, Value};

/// Mirrors `syon_parser::ErrorCode` as a Python-visible enum (`syon.ErrorCode`).
///
/// Kept as a separate, explicitly-converted type rather than exposing the core
/// enum directly, so `syon-parser` stays free of a PyO3 dependency -- the same
/// per-language mirroring the Go implementation uses (see
/// `design/architecture/0004-independent-go-implementation.syon`).
///
/// The `From` impl below is exhaustive on purpose: adding a variant to
/// `syon_parser::ErrorCode` without mirroring it here is a compile error, not a
/// silently missing constant.
#[pyclass(module = "syon", name = "ErrorCode")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyErrorCode {
    #[pyo3(name = "INVALID_UTF8")]
    InvalidUtf8 = 1,
    #[pyo3(name = "TAB_IN_INDENTATION")]
    TabInIndentation = 2,
    #[pyo3(name = "UNEXPECTED_CONTENT")]
    UnexpectedContent = 3,
    #[pyo3(name = "INDENT_NOT_MULTIPLE_OF_STEP")]
    IndentNotMultipleOfStep = 4,
    #[pyo3(name = "DECODE_TYPE_MISMATCH")]
    DecodeTypeMismatch = 11,
    #[pyo3(name = "DECODE_SHAPE_MISMATCH")]
    DecodeShapeMismatch = 12,
    #[pyo3(name = "MALFORMED_STRUCTURE")]
    MalformedStructure = 90,

    #[pyo3(name = "KEY_STARTS_WITH_OPERATOR")]
    KeyStartsWithOperator = 101,
    #[pyo3(name = "EMPTY_KEY")]
    EmptyKey = 102,
    #[pyo3(name = "DUPLICATE_KEY")]
    DuplicateKey = 103,
    #[pyo3(name = "EXPLICIT_TAG")]
    ExplicitTag = 111,
    #[pyo3(name = "ANCHOR")]
    Anchor = 112,
    #[pyo3(name = "ALIAS")]
    Alias = 113,
    #[pyo3(name = "FLOW_MAPPING")]
    FlowMapping = 114,
    #[pyo3(name = "FLOW_SEQUENCE")]
    FlowSequence = 115,
    #[pyo3(name = "COMPLEX_KEY")]
    ComplexKey = 116,
    #[pyo3(name = "DOCUMENT_START_MARKER")]
    DocumentStartMarker = 117,
    #[pyo3(name = "DOCUMENT_END_MARKER")]
    DocumentEndMarker = 118,
    #[pyo3(name = "LITERAL_BLOCK_REMOVED")]
    LiteralBlockRemoved = 119,
    #[pyo3(name = "UNTERMINATED_QUOTED_STRING")]
    UnterminatedQuotedString = 121,
    #[pyo3(name = "COMPACT_BLOCK_SCALAR_NEEDS_OPTION")]
    CompactBlockScalarNeedsOption = 131,
    #[pyo3(name = "SEQUENCE_ITEM_MIXES_MAPPING_AND_BLOCK")]
    SequenceItemMixesMappingAndBlock = 132,
    #[pyo3(name = "SEQUENCE_ITEM_INLINE_TEXT_AND_BLOCK")]
    SequenceItemInlineTextAndBlock = 133,

    #[pyo3(name = "FENCE_INFO_STRING_MALFORMED")]
    FenceInfoStringMalformed = 201,
    #[pyo3(name = "UNTERMINATED_FENCE")]
    UnterminatedFence = 202,
}

#[pymethods]
impl PyErrorCode {
    /// The numeric value, e.g. `202`.
    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    fn __str__(&self) -> String {
        format!("SYON-{:03}", *self as u16)
    }

    fn __repr__(&self) -> String {
        format!("<ErrorCode.{}: {}>", self.name(), *self as u16)
    }

    /// The variant name, e.g. `"UNTERMINATED_FENCE"`.
    #[getter]
    fn name(&self) -> &'static str {
        match self {
            PyErrorCode::InvalidUtf8 => "INVALID_UTF8",
            PyErrorCode::TabInIndentation => "TAB_IN_INDENTATION",
            PyErrorCode::UnexpectedContent => "UNEXPECTED_CONTENT",
            PyErrorCode::IndentNotMultipleOfStep => "INDENT_NOT_MULTIPLE_OF_STEP",
            PyErrorCode::DecodeTypeMismatch => "DECODE_TYPE_MISMATCH",
            PyErrorCode::DecodeShapeMismatch => "DECODE_SHAPE_MISMATCH",
            PyErrorCode::MalformedStructure => "MALFORMED_STRUCTURE",
            PyErrorCode::KeyStartsWithOperator => "KEY_STARTS_WITH_OPERATOR",
            PyErrorCode::EmptyKey => "EMPTY_KEY",
            PyErrorCode::DuplicateKey => "DUPLICATE_KEY",
            PyErrorCode::ExplicitTag => "EXPLICIT_TAG",
            PyErrorCode::Anchor => "ANCHOR",
            PyErrorCode::Alias => "ALIAS",
            PyErrorCode::FlowMapping => "FLOW_MAPPING",
            PyErrorCode::FlowSequence => "FLOW_SEQUENCE",
            PyErrorCode::ComplexKey => "COMPLEX_KEY",
            PyErrorCode::DocumentStartMarker => "DOCUMENT_START_MARKER",
            PyErrorCode::DocumentEndMarker => "DOCUMENT_END_MARKER",
            PyErrorCode::LiteralBlockRemoved => "LITERAL_BLOCK_REMOVED",
            PyErrorCode::UnterminatedQuotedString => "UNTERMINATED_QUOTED_STRING",
            PyErrorCode::CompactBlockScalarNeedsOption => "COMPACT_BLOCK_SCALAR_NEEDS_OPTION",
            PyErrorCode::SequenceItemMixesMappingAndBlock => {
                "SEQUENCE_ITEM_MIXES_MAPPING_AND_BLOCK"
            }
            PyErrorCode::SequenceItemInlineTextAndBlock => "SEQUENCE_ITEM_INLINE_TEXT_AND_BLOCK",
            PyErrorCode::FenceInfoStringMalformed => "FENCE_INFO_STRING_MALFORMED",
            PyErrorCode::UnterminatedFence => "UNTERMINATED_FENCE",
        }
    }
}

impl From<ErrorCode> for PyErrorCode {
    fn from(c: ErrorCode) -> Self {
        match c {
            ErrorCode::InvalidUtf8 => PyErrorCode::InvalidUtf8,
            ErrorCode::TabInIndentation => PyErrorCode::TabInIndentation,
            ErrorCode::UnexpectedContent => PyErrorCode::UnexpectedContent,
            ErrorCode::IndentNotMultipleOfStep => PyErrorCode::IndentNotMultipleOfStep,
            ErrorCode::DecodeTypeMismatch => PyErrorCode::DecodeTypeMismatch,
            ErrorCode::DecodeShapeMismatch => PyErrorCode::DecodeShapeMismatch,
            ErrorCode::MalformedStructure => PyErrorCode::MalformedStructure,
            ErrorCode::KeyStartsWithOperator => PyErrorCode::KeyStartsWithOperator,
            ErrorCode::EmptyKey => PyErrorCode::EmptyKey,
            ErrorCode::DuplicateKey => PyErrorCode::DuplicateKey,
            ErrorCode::ExplicitTag => PyErrorCode::ExplicitTag,
            ErrorCode::Anchor => PyErrorCode::Anchor,
            ErrorCode::Alias => PyErrorCode::Alias,
            ErrorCode::FlowMapping => PyErrorCode::FlowMapping,
            ErrorCode::FlowSequence => PyErrorCode::FlowSequence,
            ErrorCode::ComplexKey => PyErrorCode::ComplexKey,
            ErrorCode::DocumentStartMarker => PyErrorCode::DocumentStartMarker,
            ErrorCode::DocumentEndMarker => PyErrorCode::DocumentEndMarker,
            ErrorCode::LiteralBlockRemoved => PyErrorCode::LiteralBlockRemoved,
            ErrorCode::UnterminatedQuotedString => PyErrorCode::UnterminatedQuotedString,
            ErrorCode::CompactBlockScalarNeedsOption => PyErrorCode::CompactBlockScalarNeedsOption,
            ErrorCode::SequenceItemMixesMappingAndBlock => {
                PyErrorCode::SequenceItemMixesMappingAndBlock
            }
            ErrorCode::SequenceItemInlineTextAndBlock => {
                PyErrorCode::SequenceItemInlineTextAndBlock
            }
            ErrorCode::FenceInfoStringMalformed => PyErrorCode::FenceInfoStringMalformed,
            ErrorCode::UnterminatedFence => PyErrorCode::UnterminatedFence,
        }
    }
}

/// A SYON parse error, raised as `syon.SyonError`.
///
/// Still an `Exception` subclass, as before, so `except syon.SyonError` and
/// `except Exception` keep working. It now also carries `.code` (a
/// `syon.ErrorCode`), `.kind` (`"forbidden"` or `"syntax"`) and `.message`
/// (the text without the code or kind prefix). `str(e)` keeps the kind word it
/// always had and gains the code in front, mirroring the Rust `Display`.
#[pyclass(extends = PyException, module = "syon")]
pub struct SyonError {
    #[pyo3(get)]
    code: PyErrorCode,
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    message: String,
}

#[pymethods]
impl SyonError {
    #[new]
    fn new(code: PyErrorCode, kind: String, message: String) -> Self {
        SyonError {
            code,
            kind,
            message,
        }
    }

    fn __str__(&self) -> String {
        let kind = if self.kind == "forbidden" {
            "forbidden"
        } else {
            "syntax error"
        };
        format!("[SYON-{:03}] {}: {}", self.code as u16, kind, self.message)
    }
}

fn syon_error(py: Python<'_>, err: syon_parser::SyonError) -> PyErr {
    let kind = match err {
        syon_parser::SyonError::Forbidden { .. } => "forbidden",
        syon_parser::SyonError::Syntax { .. } => "syntax",
    };
    let e = SyonError::new(
        err.code().into(),
        kind.to_string(),
        err.message().to_string(),
    );
    match Py::new(py, e) {
        Ok(instance) => PyErr::from_value_bound(instance.into_bound(py).into_any()),
        Err(e) => e,
    }
}

fn value_to_py(py: Python<'_>, val: &Value) -> PyResult<PyObject> {
    match val {
        Value::Scalar(s) => Ok(s.into_py(py)),
        Value::LiteralBlock(s) => Ok(s.into_py(py)),
        Value::Mapping(entries) => {
            let dict = PyDict::new_bound(py);
            for entry in entries {
                let v = value_to_py(py, &entry.value)?;
                dict.set_item(&entry.key, v)?;
            }
            Ok(dict.into())
        }
        Value::Sequence(items) => {
            let list = PyList::empty_bound(py);
            for item in items {
                list.append(value_to_py(py, &item.value)?)?;
            }
            Ok(list.into())
        }
    }
}

#[pyfunction]
fn parse(py: Python<'_>, input: &str) -> PyResult<PyObject> {
    let file = syon_parser::parse(input).map_err(|e| syon_error(py, e))?;
    let first = file
        .documents
        .into_iter()
        .next()
        .ok_or_else(|| PyException::new_err("no documents"))?;
    value_to_py(py, &first.body)
}

// ---------------------------------------------------------------------------
// parse_documents — per-fence extraction with metadata
// ---------------------------------------------------------------------------

struct FenceSegment {
    /// Reconstructed full path, e.g. "config/service.syon". None for unfenced.
    full_path: Option<String>,
    /// Format portion only, e.g. "syon". None for unfenced.
    format: Option<String>,
    content: String,
}

/// Split `input` into fence segments without running the SYON parser.
/// Each ```` ``` ```` line that contains a `path.format` starts a new fence;
/// a bare ```` ``` ```` closes one. Implicit close happens when a new fence
/// open is encountered while already inside a fence.
fn extract_fence_segments(input: &str) -> Vec<FenceSegment> {
    let mut segments: Vec<FenceSegment> = Vec::new();
    let mut unfenced = String::new();
    let mut cur_full_path: Option<String> = None;
    let mut cur_format: Option<String> = None;
    let mut cur_content = String::new();
    let mut in_fence = false;

    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("```") {
            if in_fence {
                segments.push(FenceSegment {
                    full_path: cur_full_path.take(),
                    format: cur_format.take(),
                    content: std::mem::take(&mut cur_content),
                });
                in_fence = false;
            }
            // Check whether this ``` line also opens a new fence (has a path.format)
            if let Some(dot) = rest.find('.') {
                let path = &rest[..dot];
                let fmt = &rest[dot + 1..];
                cur_full_path = Some(format!("{path}.{fmt}"));
                cur_format = Some(fmt.to_string());
                cur_content = String::new();
                in_fence = true;
            }
        } else if in_fence {
            cur_content.push_str(line);
            cur_content.push('\n');
        } else {
            unfenced.push_str(line);
            unfenced.push('\n');
        }
    }

    // Flush an open fence at EOF
    if in_fence {
        segments.push(FenceSegment {
            full_path: cur_full_path,
            format: cur_format,
            content: cur_content,
        });
    }

    // Include unfenced content if present, or if there were no fences at all
    if !unfenced.trim().is_empty() || segments.is_empty() {
        segments.insert(
            0,
            FenceSegment {
                full_path: None,
                format: None,
                content: unfenced,
            },
        );
    }

    segments
}

#[pyfunction]
fn parse_documents(py: Python<'_>, input: &str) -> PyResult<PyObject> {
    let segments = extract_fence_segments(input);
    let list = PyList::empty_bound(py);

    for seg in &segments {
        let dict = PyDict::new_bound(py);

        // Metadata keys
        if let Some(path) = &seg.full_path {
            dict.set_item("__path__", path)?;
        }
        if let Some(fmt) = &seg.format {
            dict.set_item("__format__", fmt)?;
        }

        // Try to parse the segment's body as SYON; non-SYON fences will error
        // and simply leave the dict with only metadata keys.
        if let Ok(file) = syon_parser::parse(&seg.content) {
            if let Some(doc) = file.documents.into_iter().next() {
                if let Value::Mapping(entries) = doc.body {
                    for entry in entries {
                        let v = value_to_py(py, &entry.value)?;
                        dict.set_item(&entry.key, v)?;
                    }
                }
            }
        }

        list.append(dict)?;
    }

    Ok(list.into())
}

#[pymodule]
fn syon(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = py;
    m.add_class::<PyErrorCode>()?;
    m.add_class::<SyonError>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_documents, m)?)?;
    Ok(())
}
