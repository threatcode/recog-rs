//! Custom error types for the Recog library
//!
//! This module provides structured error handling for all Recog operations,
//! replacing generic `Box<dyn std::error::Error>` with specific, actionable error types.

use thiserror::Error;

/// Main error type for the Recog library
#[derive(Error, Debug)]
pub enum RecogError {
    /// Errors related to XML parsing and fingerprint loading
    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::Error),

    /// Errors related to regular expression compilation or matching
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Errors related to base64 encoding/decoding
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Errors related to file I/O operations
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors related to UTF-8 string conversion
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Errors related to JSON serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Errors related to invalid fingerprint data
    #[error("Invalid fingerprint data: {message}")]
    InvalidFingerprintData { message: String },

    /// Errors related to parameter processing
    #[error("Parameter error: {message}")]
    Parameter { message: String },

    /// Errors related to pattern matching
    #[error("Matching error: {message}")]
    Matching { message: String },

    /// Errors related to configuration
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Custom errors with context
    #[error("Error: {message}")]
    Custom { message: String },
}

impl From<quick_xml::DeError> for RecogError {
    fn from(err: quick_xml::DeError) -> Self {
        // Convert DeError to a string and wrap in custom error
        RecogError::custom(format!("XML deserialization error: {}", err))
    }
}

impl RecogError {
    /// Create a custom error with a message
    pub fn custom<S: Into<String>>(message: S) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }

    /// Create an invalid fingerprint data error
    pub fn invalid_fingerprint_data<S: Into<String>>(message: S) -> Self {
        Self::InvalidFingerprintData {
            message: message.into(),
        }
    }

    /// Create a parameter error
    pub fn parameter<S: Into<String>>(message: S) -> Self {
        Self::Parameter {
            message: message.into(),
        }
    }

    /// Create a matching error
    pub fn matching<S: Into<String>>(message: S) -> Self {
        Self::Matching {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
}

/// Result type alias for Recog operations
pub type RecogResult<T> = Result<T, RecogError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let custom_error = RecogError::custom("test error");
        assert!(matches!(custom_error, RecogError::Custom { .. }));

        let fingerprint_error = RecogError::invalid_fingerprint_data("invalid pattern");
        assert!(matches!(fingerprint_error, RecogError::InvalidFingerprintData { .. }));

        let param_error = RecogError::parameter("missing parameter");
        assert!(matches!(param_error, RecogError::Parameter { .. }));

        let matching_error = RecogError::matching("no match found");
        assert!(matches!(matching_error, RecogError::Matching { .. }));

        let config_error = RecogError::configuration("invalid config");
        assert!(matches!(config_error, RecogError::Configuration { .. }));
    }

    #[test]
    fn test_error_display() {
        let error = RecogError::custom("test message");
        assert_eq!(error.to_string(), "Error: test message");
    }

    #[test]
    fn test_result_alias() {
        fn returns_result() -> RecogResult<String> {
            Ok("success".to_string())
        }

        fn returns_error() -> RecogResult<String> {
            Err(RecogError::custom("failed"))
        }

        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());
    }
}
