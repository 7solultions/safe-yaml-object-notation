use std::fmt;

use crate::error_code::ErrorCode;

#[derive(Debug, Clone, PartialEq)]
pub enum SyonError {
    /// A YAML construct that SYON explicitly forbids.
    Forbidden { code: ErrorCode, message: String },
    /// A low-level syntax / parse error.
    Syntax { code: ErrorCode, message: String },
}

impl SyonError {
    /// Construct a [`SyonError::Forbidden`].
    pub fn forbidden(code: ErrorCode, message: impl Into<String>) -> Self {
        SyonError::Forbidden {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`SyonError::Syntax`].
    pub fn syntax(code: ErrorCode, message: impl Into<String>) -> Self {
        SyonError::Syntax {
            code,
            message: message.into(),
        }
    }

    /// The numeric code, stable across message rewordings.
    pub fn code(&self) -> ErrorCode {
        match self {
            SyonError::Forbidden { code, .. } | SyonError::Syntax { code, .. } => *code,
        }
    }

    /// The human-readable message, without the code or kind prefix.
    pub fn message(&self) -> &str {
        match self {
            SyonError::Forbidden { message, .. } | SyonError::Syntax { message, .. } => message,
        }
    }
}

impl fmt::Display for SyonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyonError::Forbidden { code, message } => write!(f, "[{code}] forbidden: {message}"),
            SyonError::Syntax { code, message } => write!(f, "[{code}] syntax error: {message}"),
        }
    }
}

impl std::error::Error for SyonError {}
