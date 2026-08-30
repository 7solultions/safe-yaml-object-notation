//! The error every shlita operation returns.
//!
//! One error type rather than one per module, because a caller reading a
//! literal, calling a function and stepping a chart wants to handle all
//! three the same way. What varies is the [`ShlitaCode`] and the context
//! string, and both are carried.

use std::fmt;

use crate::error_code::ShlitaCode;

/// A shlita failure: what went wrong, where, and on what.
///
/// `context` names the thing that failed -- a type name when a literal was
/// read, a function name when one was called, a block's local id when a
/// network was loaded. It is the answer to "where", and it is always present
/// because an error that does not say where is an error a reader has to
/// bisect for.
///
/// `subject` is the text or value that failed, verbatim, and is empty when
/// there is none. It is truncated by [`ShlitaError::subject_for_display`]
/// rather than at construction, so a caller that wants the whole thing keeps
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShlitaError {
    code: ShlitaCode,
    context: String,
    subject: String,
    message: String,
}

impl ShlitaError {
    pub fn new(
        code: ShlitaCode,
        context: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        ShlitaError {
            code,
            context: context.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }

    /// An error with no failing text to quote -- a missing input, a chart
    /// with two initial steps.
    pub fn at(code: ShlitaCode, context: impl Into<String>, message: impl Into<String>) -> Self {
        ShlitaError::new(code, context, "", message)
    }

    /// The numeric code, stable across message rewordings.
    pub fn code(&self) -> ShlitaCode {
        self.code
    }

    /// The type, function, block or step the failure belongs to.
    pub fn context(&self) -> &str {
        &self.context
    }

    /// The failing text, verbatim and untruncated. Empty when there is none.
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The human-readable explanation, without the code or the subject.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The subject, shortened for a one-line message. Long subjects are cut
    /// at 40 characters -- a pasted document should not become an error
    /// message.
    pub fn subject_for_display(&self) -> String {
        const LIMIT: usize = 40;
        if self.subject.chars().count() <= LIMIT {
            return self.subject.clone();
        }
        let head: String = self.subject.chars().take(LIMIT).collect();
        format!("{head}...")
    }
}

impl fmt::Display for ShlitaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.context, self.message)?;
        if !self.subject.is_empty() {
            write!(f, " (reading {:?})", self.subject_for_display())?;
        }
        Ok(())
    }
}

impl std::error::Error for ShlitaError {}

/// The result of every fallible shlita operation.
pub type Result<T> = std::result::Result<T, ShlitaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_carries_code_context_and_subject() {
        let err = ShlitaError::new(
            ShlitaCode::IntegerOutOfRange,
            "SINT",
            "300",
            "the range of SINT is -128..=127",
        );
        assert_eq!(
            err.to_string(),
            "[SHLITA-712] SINT: the range of SINT is -128..=127 (reading \"300\")"
        );
    }

    #[test]
    fn an_error_with_no_subject_does_not_quote_an_empty_one() {
        let err = ShlitaError::at(ShlitaCode::MissingInput, "TON", "PT was not supplied");
        assert_eq!(err.to_string(), "[SHLITA-751] TON: PT was not supplied");
    }

    #[test]
    fn long_subjects_are_truncated_for_display_only() {
        let long = "9".repeat(100);
        let err = ShlitaError::new(ShlitaCode::NotAnInteger, "INT", &long, "too long");
        assert_eq!(err.subject(), long);
        assert_eq!(err.subject_for_display(), format!("{}...", "9".repeat(40)));
    }
}
