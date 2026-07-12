use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use syon_parser::{ErrorCode, Value};

/// Mirrors `syon_parser::ErrorCode` as a Python-visible enum (`syon.ErrorCode`).
/// Kept as a separate, explicitly-converted type here rather than exposing
/// `syon_parser::ErrorCode` directly, so the core crate stays free of a PyO3
/// dependency -- matching the project's other independent-per-language
/// mirroring (see docs/decisions/0004-independent-go-implementation.syon).
#[pyclass(module = "syon", name = "ErrorCode")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyErrorCode {
    #[pyo3(name = "INVALID_UTF8")]
    InvalidUtf8 = 1,
    #[pyo3(name = "TAB_IN_INDENTATION")]
    TabInIndentation = 2,
    #[pyo3(name = "UNEXPECTED_TRAILING_CONTENT")]
    UnexpectedTrailingContent = 3,
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
    #[pyo3(name = "UNTERMINATED_QUOTED_STRING")]
    UnterminatedQuotedString = 121,

    #[pyo3(name = "UNTERMINATED_LITERAL_BLOCK")]
    UnterminatedLiteralBlock = 202,
    #[pyo3(name = "LITERAL_EXPLICIT_TAG")]
    LiteralExplicitTag = 211,
    #[pyo3(name = "LITERAL_ANCHOR")]
    LiteralAnchor = 212,
    #[pyo3(name = "LITERAL_ALIAS")]
    LiteralAlias = 213,
    #[pyo3(name = "LITERAL_FLOW_MAPPING")]
    LiteralFlowMapping = 214,
    #[pyo3(name = "LITERAL_FLOW_SEQUENCE")]
    LiteralFlowSequence = 215,
    #[pyo3(name = "LITERAL_COMPLEX_KEY")]
    LiteralComplexKey = 216,
    #[pyo3(name = "LITERAL_DOCUMENT_START_MARKER")]
    LiteralDocumentStartMarker = 217,
    #[pyo3(name = "LITERAL_DOCUMENT_END_MARKER")]
    LiteralDocumentEndMarker = 218,

    #[pyo3(name = "FENCE_INFO_STRING_MALFORMED")]
    FenceInfoStringMalformed = 301,
    #[pyo3(name = "UNTERMINATED_FENCE")]
    UnterminatedFence = 302,
    #[pyo3(name = "FENCE_EXPLICIT_TAG")]
    FenceExplicitTag = 311,
    #[pyo3(name = "FENCE_ANCHOR")]
    FenceAnchor = 312,
    #[pyo3(name = "FENCE_ALIAS")]
    FenceAlias = 313,
    #[pyo3(name = "FENCE_FLOW_MAPPING")]
    FenceFlowMapping = 314,
    #[pyo3(name = "FENCE_FLOW_SEQUENCE")]
    FenceFlowSequence = 315,
    #[pyo3(name = "FENCE_COMPLEX_KEY")]
    FenceComplexKey = 316,
    #[pyo3(name = "FENCE_DOCUMENT_START_MARKER")]
    FenceDocumentStartMarker = 317,
    #[pyo3(name = "FENCE_DOCUMENT_END_MARKER")]
    FenceDocumentEndMarker = 318,
}

#[pymethods]
impl PyErrorCode {
    fn __repr__(&self) -> String {
        format!("ErrorCode.{}", self.variant_name())
    }
}

impl PyErrorCode {
    fn variant_name(&self) -> &'static str {
        match self {
            PyErrorCode::InvalidUtf8 => "INVALID_UTF8",
            PyErrorCode::TabInIndentation => "TAB_IN_INDENTATION",
            PyErrorCode::UnexpectedTrailingContent => "UNEXPECTED_TRAILING_CONTENT",
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
            PyErrorCode::UnterminatedQuotedString => "UNTERMINATED_QUOTED_STRING",
            PyErrorCode::UnterminatedLiteralBlock => "UNTERMINATED_LITERAL_BLOCK",
            PyErrorCode::LiteralExplicitTag => "LITERAL_EXPLICIT_TAG",
            PyErrorCode::LiteralAnchor => "LITERAL_ANCHOR",
            PyErrorCode::LiteralAlias => "LITERAL_ALIAS",
            PyErrorCode::LiteralFlowMapping => "LITERAL_FLOW_MAPPING",
            PyErrorCode::LiteralFlowSequence => "LITERAL_FLOW_SEQUENCE",
            PyErrorCode::LiteralComplexKey => "LITERAL_COMPLEX_KEY",
            PyErrorCode::LiteralDocumentStartMarker => "LITERAL_DOCUMENT_START_MARKER",
            PyErrorCode::LiteralDocumentEndMarker => "LITERAL_DOCUMENT_END_MARKER",
            PyErrorCode::FenceInfoStringMalformed => "FENCE_INFO_STRING_MALFORMED",
            PyErrorCode::UnterminatedFence => "UNTERMINATED_FENCE",
            PyErrorCode::FenceExplicitTag => "FENCE_EXPLICIT_TAG",
            PyErrorCode::FenceAnchor => "FENCE_ANCHOR",
            PyErrorCode::FenceAlias => "FENCE_ALIAS",
            PyErrorCode::FenceFlowMapping => "FENCE_FLOW_MAPPING",
            PyErrorCode::FenceFlowSequence => "FENCE_FLOW_SEQUENCE",
            PyErrorCode::FenceComplexKey => "FENCE_COMPLEX_KEY",
            PyErrorCode::FenceDocumentStartMarker => "FENCE_DOCUMENT_START_MARKER",
            PyErrorCode::FenceDocumentEndMarker => "FENCE_DOCUMENT_END_MARKER",
        }
    }
}

impl From<ErrorCode> for PyErrorCode {
    fn from(c: ErrorCode) -> Self {
        match c {
            ErrorCode::InvalidUtf8 => PyErrorCode::InvalidUtf8,
            ErrorCode::TabInIndentation => PyErrorCode::TabInIndentation,
            ErrorCode::UnexpectedTrailingContent => PyErrorCode::UnexpectedTrailingContent,
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
            ErrorCode::UnterminatedQuotedString => PyErrorCode::UnterminatedQuotedString,
            ErrorCode::UnterminatedLiteralBlock => PyErrorCode::UnterminatedLiteralBlock,
            ErrorCode::LiteralExplicitTag => PyErrorCode::LiteralExplicitTag,
            ErrorCode::LiteralAnchor => PyErrorCode::LiteralAnchor,
            ErrorCode::LiteralAlias => PyErrorCode::LiteralAlias,
            ErrorCode::LiteralFlowMapping => PyErrorCode::LiteralFlowMapping,
            ErrorCode::LiteralFlowSequence => PyErrorCode::LiteralFlowSequence,
            ErrorCode::LiteralComplexKey => PyErrorCode::LiteralComplexKey,
            ErrorCode::LiteralDocumentStartMarker => PyErrorCode::LiteralDocumentStartMarker,
            ErrorCode::LiteralDocumentEndMarker => PyErrorCode::LiteralDocumentEndMarker,
            ErrorCode::FenceInfoStringMalformed => PyErrorCode::FenceInfoStringMalformed,
            ErrorCode::UnterminatedFence => PyErrorCode::UnterminatedFence,
            ErrorCode::FenceExplicitTag => PyErrorCode::FenceExplicitTag,
            ErrorCode::FenceAnchor => PyErrorCode::FenceAnchor,
            ErrorCode::FenceAlias => PyErrorCode::FenceAlias,
            ErrorCode::FenceFlowMapping => PyErrorCode::FenceFlowMapping,
            ErrorCode::FenceFlowSequence => PyErrorCode::FenceFlowSequence,
            ErrorCode::FenceComplexKey => PyErrorCode::FenceComplexKey,
            ErrorCode::FenceDocumentStartMarker => PyErrorCode::FenceDocumentStartMarker,
            ErrorCode::FenceDocumentEndMarker => PyErrorCode::FenceDocumentEndMarker,
        }
    }
}

/// A SYON parse error, raised as `syon.SyonError` -- a `ValueError` subclass
/// (so existing `except ValueError` code keeps working) carrying `.code`
/// (a `syon.ErrorCode`) and `.message` (the human-readable text).
#[pyclass(extends = PyValueError, module = "syon")]
pub struct SyonError {
    #[pyo3(get)]
    code: PyErrorCode,
    #[pyo3(get)]
    message: String,
}

#[pymethods]
impl SyonError {
    #[new]
    fn new(code: PyErrorCode, message: String) -> Self {
        SyonError { code, message }
    }

    fn __str__(&self) -> String {
        format!("[SYON-{:03}] {}", self.code as u16, self.message)
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

fn syon_error(py: Python<'_>, err: syon_parser::SyonError) -> PyErr {
    let code = PyErrorCode::from(err.code());
    let message = err.message().to_string();
    match Py::new(py, SyonError::new(code, message)) {
        Ok(instance) => PyErr::from_value_bound(instance.into_bound(py).into_any()),
        Err(e) => e,
    }
}

#[pyfunction]
fn parse(py: Python<'_>, input: &str) -> PyResult<PyObject> {
    let file = syon_parser::parse(input).map_err(|e| syon_error(py, e))?;
    let first = file
        .documents
        .into_iter()
        .next()
        .ok_or_else(|| PyValueError::new_err("no documents"))?;
    value_to_py(py, &first.body)
}

#[pymodule]
fn syon(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_class::<PyErrorCode>()?;
    m.add_class::<SyonError>()?;
    Ok(())
}
