use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter definition for extraction from regex captures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    /// Position in the regex capture group (1-indexed)
    pub pos: usize,
    /// Name of the parameter
    pub name: String,
    /// Optional default value
    pub value: Option<String>,
}

impl Param {
    /// Create a new parameter definition
    pub fn new(pos: usize, name: String) -> Self {
        Param {
            pos,
            name,
            value: None,
        }
    }

    /// Create a parameter with a default value
    pub fn with_value(pos: usize, name: String, value: String) -> Self {
        Param {
            pos,
            name,
            value: Some(value),
        }
    }
}

/// Handle parameter interpolation with support for {param} syntax
pub struct ParamInterpolator {
    /// Temporary parameters that shouldn't be emitted in final results
    temp_params: Vec<String>,
}

impl ParamInterpolator {
    /// Create a new interpolator
    pub fn new() -> Self {
        ParamInterpolator {
            temp_params: Vec::new(),
        }
    }

    /// Add a temporary parameter (prefixed with _tmp.)
    pub fn add_temp_param(&mut self, name: &str) {
        self.temp_params.push(name.to_string());
    }

    /// Interpolate parameters into a template string
    pub fn interpolate(&self, template: &str, params: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        // Replace {param_name} patterns
        for (param_name, param_value) in params {
            let pattern = format!("{{{}}}", param_name);
            result = result.replace(&pattern, param_value);
        }

        // Remove any remaining {param_name} patterns
        let re = regex::Regex::new(r"\{[^}]+\}").unwrap();
        result = re.replace_all(&result, "").to_string();

        result
    }

    /// Filter out temporary parameters from results
    pub fn filter_temp_params(&self, params: &mut HashMap<String, String>) {
        params.retain(|name, _| !self.temp_params.contains(name) && !name.starts_with("_tmp."));
    }

    /// Process CPE (Common Platform Enumeration) parameters
    pub fn process_cpe_params(&self, params: &mut HashMap<String, String>) {
        // Handle CPE-specific parameter processing
        // This would implement CPE field mapping and formatting

        // Filter out temporary parameters that shouldn't appear in CPE
        self.filter_temp_params(params);

        // Add CPE-specific transformations here if needed
        // For example, mapping hw.product to cpe.vendor, etc.
    }
}

impl Default for ParamInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_creation() {
        let param = Param::new(1, "version".to_string());
        assert_eq!(param.pos, 1);
        assert_eq!(param.name, "version");
        assert!(param.value.is_none());

        let param_with_value = Param::with_value(2, "product".to_string(), "Apache".to_string());
        assert_eq!(param_with_value.pos, 2);
        assert_eq!(param_with_value.name, "product");
        assert_eq!(param_with_value.value, Some("Apache".to_string()));
    }

    #[test]
    fn test_interpolation() {
        let interpolator = ParamInterpolator::new();
        let mut params = HashMap::new();
        params.insert("version".to_string(), "2.4.41".to_string());
        params.insert("product".to_string(), "Apache".to_string());

        let template = "Server: {product}/{version}";
        let result = interpolator.interpolate(template, &params);
        assert_eq!(result, "Server: Apache/2.4.41");
    }

    #[test]
    fn test_temp_params() {
        let mut interpolator = ParamInterpolator::new();
        interpolator.add_temp_param("_tmp.os");

        let mut params = HashMap::new();
        params.insert("product".to_string(), "Apache".to_string());
        params.insert("_tmp.os".to_string(), "Linux".to_string());
        params.insert("_tmp.version".to_string(), "2.4".to_string());

        interpolator.filter_temp_params(&mut params);

        assert_eq!(params.len(), 1);
        assert_eq!(params.get("product"), Some(&"Apache".to_string()));
        assert!(!params.contains_key("_tmp.os"));
    }
}
