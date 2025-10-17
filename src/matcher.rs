use crate::error::RecogResult;
use crate::fingerprint::{Fingerprint, FingerprintDatabase};
use crate::params::ParamInterpolator;
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;

/// Result of a fingerprint match
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The fingerprint that matched
    pub fingerprint: Fingerprint,
    /// Captured parameters
    pub params: HashMap<String, String>,
    /// Match score/confidence (for future use)
    pub score: f32,
}

impl MatchResult {
    /// Create a new match result
    pub fn new(fingerprint: Fingerprint, params: HashMap<String, String>) -> Self {
        MatchResult {
            fingerprint,
            params,
            score: 1.0, // Default score
        }
    }

    /// Convert to JSON for output
    pub fn to_json(&self) -> RecogResult<String> {
        let mut result = serde_json::Map::new();
        result.insert(
            "description".to_string(),
            serde_json::Value::String(self.fingerprint.description.clone()),
        );
        result.insert("params".to_string(), serde_json::to_value(&self.params)?);

        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Matcher engine for processing text against fingerprints
pub struct Matcher {
    /// Database of fingerprints
    db: FingerprintDatabase,
    /// Parameter interpolator
    interpolator: ParamInterpolator,
}

impl Matcher {
    /// Create a new matcher with a fingerprint database
    pub fn new(db: FingerprintDatabase) -> Self {
        Matcher {
            db,
            interpolator: ParamInterpolator::new(),
        }
    }

    /// Create a matcher from a database reference (consuming it)
    pub fn from_db(db: FingerprintDatabase) -> Self {
        Self::new(db)
    }

    /// Match text against all fingerprints and return all matches
    pub fn match_text(&self, text: &str) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for fingerprint in &self.db.fingerprints {
            if let Some(mut params) = fingerprint.matches(text) {
                // Apply parameter interpolation and filtering
                self.interpolator.process_cpe_params(&mut params);

                results.push(MatchResult::new(fingerprint.clone(), params));
            }
        }

        results
    }

    /// Match text and return the best match (first one found)
    pub fn match_text_best(&self, text: &str) -> Option<MatchResult> {
        self.match_text(text).into_iter().next()
    }

    /// Match base64-encoded text
    pub fn match_base64(&self, base64_text: &str) -> RecogResult<Vec<MatchResult>> {
        let decoded = general_purpose::STANDARD.decode(base64_text)?;
        let text = String::from_utf8(decoded)?;

        Ok(self.match_text(&text))
    }

    /// Match with multiple texts (for batch processing)
    pub fn match_batch(&self, texts: &[String]) -> Vec<Vec<MatchResult>> {
        texts.iter().map(|text| self.match_text(text)).collect()
    }

    /// Get the underlying fingerprint database
    pub fn database(&self) -> &FingerprintDatabase {
        &self.db
    }

    /// Get the parameter interpolator
    pub fn interpolator(&self) -> &ParamInterpolator {
        &self.interpolator
    }

    /// Get a mutable reference to the interpolator for configuration
    pub fn interpolator_mut(&mut self) -> &mut ParamInterpolator {
        &mut self.interpolator
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new(FingerprintDatabase::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_fingerprints_from_xml;

    #[test]
    fn test_basic_matching() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="Apache/(\d+\.\d+)" description="Apache HTTP Server">
                    <param pos="1" name="version"/>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let matcher = Matcher::new(db);

        let results = matcher.match_text("Server: Apache/2.4.41");
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert_eq!(result.fingerprint.description, "Apache HTTP Server");
        assert_eq!(result.params.get("version"), Some(&"2.4.41".to_string()));
    }

    #[test]
    fn test_no_match() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="Apache/(\d+\.\d+)" description="Apache HTTP Server">
                    <param pos="1" name="version"/>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let matcher = Matcher::new(db);

        let results = matcher.match_text("nginx/1.20.0");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_base64_matching() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="test" description="Test pattern">
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let matcher = Matcher::new(db);

        // Test base64 decoding and matching
        let results = matcher.match_base64("dGVzdA==").unwrap(); // "test" in base64
        assert_eq!(results.len(), 1);
    }
}
