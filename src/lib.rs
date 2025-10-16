//! Recog - Rust-native library for fingerprint-based recognition
//!
//! This library provides functionality for parsing XML fingerprint databases,
//! matching text patterns against fingerprints, and extracting parameters.
//! It's designed to be a high-performance, safe alternative to other Recog implementations.

pub mod async_loader;
pub mod cli;
pub mod comprehensive_tests;
pub mod error;
pub mod fingerprint;
pub mod loader;
pub mod matcher;
pub mod params;
pub mod plugin;

#[cfg(feature = "async")]
pub mod async_loader;

// Re-export main types for convenience
#[cfg(feature = "async")]
pub use async_loader::{
    load_fingerprints_from_file_async, load_fingerprints_from_xml_async,
    load_multiple_databases_async, StreamingXmlLoader,
};
pub use error::{RecogError, RecogResult};
pub use fingerprint::{Example, Fingerprint, FingerprintDatabase};
pub use loader::{load_fingerprints_from_file, load_fingerprints_from_xml};
pub use matcher::{MatchResult, Matcher};
pub use params::{Param, ParamInterpolator};
pub use plugin::{
    FuzzyPatternMatcher, PatternMatchResult, PatternMatcher, PatternMatcherRegistry,
    PluginFingerprint, RegexPatternMatcher, StringPatternMatcher,
};
