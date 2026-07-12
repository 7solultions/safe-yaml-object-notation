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
    pub fn code(&self) -> ErrorCode {
        match self {
            SyonError::Forbidden { code, .. } | SyonError::Syntax { code, .. } => *code,
        }
    }

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
