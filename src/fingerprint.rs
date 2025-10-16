use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core data structures for Recog fingerprints
use crate::{error::RecogResult, params::Param};

/// A fingerprint pattern for matching against network banners
#[derive(Debug, Clone)]
pub struct Fingerprint {
    /// Regex pattern for matching
    pub pattern: Regex,
    /// Human-readable description of what this fingerprint identifies
    pub description: String,
    /// Test examples for this fingerprint
    pub examples: Vec<Example>,
    /// Parameters that can be extracted from matches
    pub params: Vec<Param>,
}

impl Fingerprint {
    /// Create a new fingerprint with a regex pattern and description
    pub fn new(pattern: &str, description: &str) -> RecogResult<Self> {
        Ok(Fingerprint {
            pattern: Regex::new(pattern)?,
            description: description.to_string(),
            examples: Vec::new(),
            params: Vec::new(),
        })
    }

    /// Add a test example to this fingerprint
    pub fn add_example(&mut self, example: Example) {
        self.examples.push(example);
    }

    /// Add a parameter definition
    pub fn add_param(&mut self, param: Param) {
        self.params.push(param);
    }

    /// Match against input text and return captured parameters
    pub fn matches(&self, text: &str) -> Option<HashMap<String, String>> {
        if let Some(captures) = self.pattern.captures(text) {
            let mut results = HashMap::new();

            // Extract parameters based on their positions
            for param in &self.params {
                if let Some(capture) = captures.get(param.pos) {
                    results.insert(param.name.clone(), capture.as_str().to_string());
                }
            }

            Some(results)
        } else {
            None
        }
    }
}

/// An example for testing a fingerprint
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

/// Collection of fingerprints loaded from XML
#[derive(Debug, Clone)]
pub struct FingerprintDatabase {
    /// All loaded fingerprints
    pub fingerprints: Vec<Fingerprint>,
}

impl FingerprintDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        FingerprintDatabase {
            fingerprints: Vec::new(),
        }
    }

    /// Add a fingerprint to the database
    pub fn add_fingerprint(&mut self, fingerprint: Fingerprint) {
        self.fingerprints.push(fingerprint);
    }

    /// Find all fingerprints that match the given text
    pub fn find_matches(&self, text: &str) -> Vec<(&Fingerprint, HashMap<String, String>)> {
        let mut matches = Vec::new();

        for fingerprint in &self.fingerprints {
            if let Some(captures) = fingerprint.matches(text) {
                matches.push((fingerprint, captures));
            }
        }

        matches
    }

    /// Find the best matching fingerprint (first match)
    pub fn find_best_match(&self, text: &str) -> Option<(&Fingerprint, HashMap<String, String>)> {
        self.find_matches(text).into_iter().next()
    }
}

impl Default for FingerprintDatabase {
    fn default() -> Self {
        Self::new()
    }
}
