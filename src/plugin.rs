//! Plugin architecture for custom pattern matchers
//!
//! This module provides a plugin system similar to the Java implementation,
//! allowing users to implement custom pattern matching engines beyond the default regex-based matcher.

use crate::error::RecogResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a pattern match operation
#[derive(Debug, Clone)]
pub struct PatternMatchResult {
    /// Whether the pattern matched
    pub matched: bool,
    /// Captured parameter values (if matched)
    pub params: HashMap<String, String>,
    /// Match confidence score (0.0 to 1.0)
    pub confidence: f32,
}

impl PatternMatchResult {
    /// Create a successful match result
    pub fn success(params: HashMap<String, String>) -> Self {
        Self {
            matched: true,
            params,
            confidence: 1.0,
        }
    }

    /// Create a failed match result
    pub fn failure() -> Self {
        Self {
            matched: false,
            params: HashMap::new(),
            confidence: 0.0,
        }
    }

    /// Create a match result with custom confidence
    pub fn with_confidence(params: HashMap<String, String>, confidence: f32) -> Self {
        Self {
            matched: !params.is_empty(),
            params,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Trait for custom pattern matchers
pub trait PatternMatcher: Send + Sync + std::fmt::Debug {
    /// Match the given text against this pattern
    fn matches(&self, text: &str) -> RecogResult<PatternMatchResult>;

    /// Get a description of this pattern matcher
    fn description(&self) -> &str;

    /// Clone this matcher for use in multiple threads
    fn clone_box(&self) -> Box<dyn PatternMatcher>;
}

/// Default regex-based pattern matcher
#[derive(Debug)]
pub struct RegexPatternMatcher {
    pattern: regex::Regex,
    description: String,
}

impl RegexPatternMatcher {
    /// Create a new regex pattern matcher
    pub fn new(pattern: &str, description: &str) -> RecogResult<Self> {
        Ok(Self {
            pattern: regex::Regex::new(pattern)?,
            description: description.to_string(),
        })
    }
}

impl PatternMatcher for RegexPatternMatcher {
    fn matches(&self, text: &str) -> RecogResult<PatternMatchResult> {
        if let Some(captures) = self.pattern.captures(text) {
            let mut params = HashMap::new();

            // Extract all capture groups as parameters
            for (i, capture) in captures.iter().enumerate().skip(1) {
                if let Some(m) = capture {
                    params.insert(format!("capture_{}", i), m.as_str().to_string());
                }
            }

            Ok(PatternMatchResult::success(params))
        } else {
            Ok(PatternMatchResult::failure())
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn clone_box(&self) -> Box<dyn PatternMatcher> {
        Box::new(Self {
            pattern: self.pattern.clone(),
            description: self.description.clone(),
        })
    }
}

/// Simple string pattern matcher (for exact matches)
#[derive(Debug)]
pub struct StringPatternMatcher {
    pattern: String,
    description: String,
}

impl StringPatternMatcher {
    /// Create a new string pattern matcher
    pub fn new(pattern: String, description: &str) -> Self {
        Self {
            pattern,
            description: description.to_string(),
        }
    }
}

impl PatternMatcher for StringPatternMatcher {
    fn matches(&self, text: &str) -> RecogResult<PatternMatchResult> {
        if text == self.pattern {
            let mut params = HashMap::new();
            params.insert("matched_string".to_string(), text.to_string());
            Ok(PatternMatchResult::success(params))
        } else {
            Ok(PatternMatchResult::failure())
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn clone_box(&self) -> Box<dyn PatternMatcher> {
        Box::new(Self {
            pattern: self.pattern.clone(),
            description: self.description.clone(),
        })
    }
}

/// Fuzzy string matcher with configurable similarity threshold
#[derive(Debug)]
pub struct FuzzyPatternMatcher {
    pattern: String,
    description: String,
    threshold: f32,
}

impl FuzzyPatternMatcher {
    /// Create a new fuzzy pattern matcher
    pub fn new(pattern: String, description: &str, threshold: f32) -> Self {
        Self {
            pattern,
            description: description.to_string(),
            threshold: threshold.clamp(0.0, 1.0),
        }
    }
}

impl PatternMatcher for FuzzyPatternMatcher {
    fn matches(&self, text: &str) -> RecogResult<PatternMatchResult> {
        let similarity = calculate_similarity(&self.pattern, text);
        if similarity >= self.threshold {
            let mut params = HashMap::new();
            params.insert("matched_string".to_string(), text.to_string());
            params.insert("similarity".to_string(), format!("{:.3}", similarity));
            Ok(PatternMatchResult::with_confidence(params, similarity))
        } else {
            Ok(PatternMatchResult::failure())
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn clone_box(&self) -> Box<dyn PatternMatcher> {
        Box::new(Self {
            pattern: self.pattern.clone(),
            description: self.description.clone(),
            threshold: self.threshold,
        })
    }
}

/// Calculate similarity between two strings using Levenshtein distance
fn calculate_similarity(s1: &str, s2: &str) -> f32 {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }

    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let max_len = len1.max(len2);
    let distance = levenshtein_distance(s1, s2);

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let len1 = chars1.len();
    let len2 = chars2.len();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };

            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

/// Plugin registry for managing custom pattern matchers
pub struct PatternMatcherRegistry {
    matchers: HashMap<String, Box<dyn PatternMatcher>>,
}

impl PatternMatcherRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            matchers: HashMap::new(),
        }
    }

    /// Register a pattern matcher with a name
    pub fn register(&mut self, name: String, matcher: Box<dyn PatternMatcher>) {
        self.matchers.insert(name, matcher);
    }

    /// Get a pattern matcher by name
    pub fn get(&self, name: &str) -> Option<&dyn PatternMatcher> {
        self.matchers.get(name).map(|m| m.as_ref())
    }

    /// List all registered matcher names
    pub fn list_matchers(&self) -> Vec<&String> {
        self.matchers.keys().collect()
    }

    /// Remove a matcher from the registry
    pub fn unregister(&mut self, name: &str) -> bool {
        self.matchers.remove(name).is_some()
    }
}

impl Default for PatternMatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced fingerprint that supports custom pattern matchers
#[derive(Debug)]
pub struct PluginFingerprint {
    /// Unique identifier for this fingerprint
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Custom pattern matcher
    pub matcher: Box<dyn PatternMatcher>,
    /// Test examples for validation
    pub examples: Vec<Example>,
    /// Parameter definitions
    pub params: Vec<crate::params::Param>,
}

impl PluginFingerprint {
    /// Create a new fingerprint with a custom matcher
    pub fn new(
        id: String,
        description: String,
        matcher: Box<dyn PatternMatcher>,
        examples: Vec<Example>,
        params: Vec<crate::params::Param>,
    ) -> Self {
        Self {
            id,
            description,
            matcher,
            examples,
            params,
        }
    }

    /// Create a fingerprint with a regex matcher
    pub fn with_regex(
        id: String,
        pattern: &str,
        description: &str,
        examples: Vec<Example>,
        params: Vec<crate::params::Param>,
    ) -> RecogResult<Self> {
        let regex_matcher = RegexPatternMatcher::new(pattern, description)?;
        Ok(Self::new(
            id,
            description.to_string(),
            Box::new(regex_matcher),
            examples,
            params,
        ))
    }

    /// Test this fingerprint against input text
    pub fn test_match(&self, text: &str) -> RecogResult<PatternMatchResult> {
        self.matcher.matches(text)
    }

    /// Validate examples against this fingerprint
    pub fn validate_examples(&self) -> RecogResult<Vec<bool>> {
        let mut results = Vec::new();

        for example in &self.examples {
            let text = if example.is_base64 {
                let decoded = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &example.value,
                )?;
                String::from_utf8(decoded)?
            } else {
                example.value.clone()
            };

            let match_result = self.test_match(&text)?;
            let is_valid = match_result.matched;
            results.push(is_valid);
        }

        Ok(results)
    }
}

/// Example for plugin fingerprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// The example value to match against
    pub value: String,
    /// Expected parameter values for this example
    pub expected_values: HashMap<String, String>,
    /// Whether this example is base64 encoded
    pub is_base64: bool,
}

impl Example {
    /// Create a new example
    pub fn new(value: String) -> Self {
        Example {
            value,
            expected_values: HashMap::new(),
            is_base64: false,
        }
    }

    /// Create a base64-encoded example
    pub fn new_base64(value: String) -> Self {
        Example {
            value,
            expected_values: HashMap::new(),
            is_base64: true,
        }
    }

    /// Add an expected parameter value
    pub fn add_expected(&mut self, name: String, value: String) {
        self.expected_values.insert(name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_matcher() {
        let matcher = RegexPatternMatcher::new(r"^Apache/(\d+\.\d+)", "Apache Server").unwrap();
        let result = matcher.matches("Apache/2.4.41").unwrap();

        assert!(result.matched);
        assert_eq!(result.params.get("capture_1"), Some(&"2.4.41".to_string()));
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_string_matcher() {
        let matcher = StringPatternMatcher::new("exact match".to_string(), "Exact match test");
        let result = matcher.matches("exact match").unwrap();

        assert!(result.matched);
        assert_eq!(
            result.params.get("matched_string"),
            Some(&"exact match".to_string())
        );
    }

    #[test]
    fn test_fuzzy_matcher() {
        let matcher = FuzzyPatternMatcher::new("apache".to_string(), "Fuzzy Apache match", 0.8);

        // Test exact match
        let result1 = matcher.matches("apache").unwrap();
        assert!(result1.matched);
        assert!(result1.confidence >= 0.8);

        // Test similar match
        let result2 = matcher.matches("apach").unwrap();
        assert!(result2.matched);
        assert!(result2.confidence < 1.0 && result2.confidence >= 0.8);

        // Test dissimilar match
        let result3 = matcher.matches("nginx").unwrap();
        assert!(!result3.matched);
    }

    #[test]
    fn test_matcher_registry() {
        let mut registry = PatternMatcherRegistry::new();

        let regex_matcher = Box::new(RegexPatternMatcher::new(r"^test", "Test matcher").unwrap());
        registry.register("regex_test".to_string(), regex_matcher);

        let string_matcher = Box::new(StringPatternMatcher::new(
            "hello".to_string(),
            "String matcher",
        ));
        registry.register("string_test".to_string(), string_matcher);

        assert_eq!(registry.list_matchers().len(), 2);
        assert!(registry.get("regex_test").is_some());
        assert!(registry.get("string_test").is_some());
        assert!(registry.get("nonexistent").is_none());

        // Test unregistration
        assert!(registry.unregister("regex_test"));
        assert_eq!(registry.list_matchers().len(), 1);
        assert!(!registry.unregister("regex_test")); // Should return false
    }

    #[test]
    fn test_plugin_fingerprint() {
        let examples = vec![Example::new("Apache/2.4.41".to_string())];

        let params = vec![crate::params::Param::new(1, "version".to_string())];

        let fingerprint = PluginFingerprint::with_regex(
            "apache_server".to_string(),
            r"^Apache/(\d+\.\d+)",
            "Apache HTTP Server",
            examples,
            params,
        )
        .unwrap();

        assert_eq!(fingerprint.id, "apache_server");
        assert_eq!(fingerprint.description, "Apache HTTP Server");

        // Test matching
        let result = fingerprint.test_match("Apache/2.4.41").unwrap();
        assert!(result.matched);
        assert_eq!(result.params.get("capture_1"), Some(&"2.4.41".to_string()));

        // Test example validation
        let validation = fingerprint.validate_examples().unwrap();
        assert_eq!(validation.len(), 1);
        assert!(validation[0]); // Should be valid
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(calculate_similarity("test", "test"), 1.0);
        assert_eq!(calculate_similarity("test", "tset"), 0.75); // 1 character different
        assert_eq!(calculate_similarity("test", "testing"), 0.8); // 3 characters different, longer string
        assert_eq!(calculate_similarity("", ""), 1.0);
        assert_eq!(calculate_similarity("test", ""), 0.0);
    }
}
