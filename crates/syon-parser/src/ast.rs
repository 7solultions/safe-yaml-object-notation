use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Scalar(String),
    Mapping(Vec<MappingEntry>),
    Sequence(Vec<SequenceItem>),
    LiteralBlock(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceItem {
    /// Source line, 1-based. Zero when unknown.
    ///
    /// Carried so a consumer can point at a line rather than at a whole
    /// file -- "unrecognised field" is far less useful without one.
    #[serde(default)]
    pub line: usize,
    pub value: Value,
    pub leading_comments: Vec<String>,
    pub trailing_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappingEntry {
    /// Source line of the key, 1-based. Zero when unknown.
    #[serde(default)]
    pub line: usize,
    pub key: String,
    pub value: Value,
    pub leading_comments: Vec<String>,
    pub trailing_comment: Option<String>,
}

/// A sub-document introduced by a ``` fence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub path: Option<String>,
    pub format: Option<String>,
    pub body: Value,
}

/// The top-level parse result: one or more documents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyonFile {
    pub documents: Vec<Document>,
}
