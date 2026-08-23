use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use syon_parser::Value;

create_exception!(syon, SyonError, PyException);

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
    let file = syon_parser::parse(input)
        .map_err(|e| SyonError::new_err(e.to_string()))?;
    let first = file
        .documents
        .into_iter()
        .next()
        .ok_or_else(|| SyonError::new_err("no documents"))?;
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
        segments.insert(0, FenceSegment {
            full_path: None,
            format: None,
            content: unfenced,
        });
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
    m.add("SyonError", py.get_type_bound::<SyonError>())?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_documents, m)?)?;
    Ok(())
}
