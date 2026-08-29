//! The error returned when a literal does not fit the type it was read at.

use std::fmt;

use crate::error_code::SheniCode;

/// A typing failure: what went wrong, at which type, on which literal.
///
/// The literal is carried so a message can quote it. It is truncated by
/// [`TypeError::literal_for_display`] rather than at construction, so a caller
/// that wants the whole thing still has it.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    code: SheniCode,
    /// The name of the type the literal was read at, e.g. `u8`.
    type_name: String,
    /// The literal that failed, verbatim.
    literal: String,
    message: String,
}

impl TypeError {
    pub fn new(
        code: SheniCode,
        type_name: impl Into<String>,
        literal: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        TypeError {
            code,
            type_name: type_name.into(),
            literal: literal.into(),
            message: message.into(),
        }
    }

    /// The numeric code, stable across message rewordings.
    pub fn code(&self) -> SheniCode {
        self.code
    }

    /// The name of the type the literal was read at.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// The failing literal, verbatim and untruncated.
    pub fn literal(&self) -> &str {
        &self.literal
    }

    /// The human-readable explanation, without the code or the literal.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The literal, shortened for a one-line message. Long literals are cut at
    /// 40 characters -- a pasted document should not become an error message.
    pub fn literal_for_display(&self) -> String {
        const LIMIT: usize = 40;
        if self.literal.chars().count() <= LIMIT {
            return self.literal.clone();
        }
        let head: String = self.literal.chars().take(LIMIT).collect();
        format!("{head}...")
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} (reading {:?})",
            self.code,
            self.type_name,
            self.message,
            self.literal_for_display()
        )
    }
}

impl std::error::Error for TypeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_carries_code_type_and_literal() {
        let err = TypeError::new(SheniCode::LeadingZero, "u8", "007", "leading zeros");
        assert_eq!(
            err.to_string(),
            "[SHENI-105] u8: leading zeros (reading \"007\")"
        );
    }

    #[test]
    fn long_literals_are_truncated_for_display_only() {
        let long = "9".repeat(100);
        let err = TypeError::new(SheniCode::IntegerOutOfRange, "u8", &long, "out of range");
        assert_eq!(err.literal(), long);
        assert_eq!(err.literal_for_display(), format!("{}...", "9".repeat(40)));
    }
}
