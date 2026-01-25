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
        // Filter out temporary parameters first
        self.filter_temp_params(params);

        // Map service/os/hw parameters to CPE components
        let vendor = params
            .get("cpe.vendor")
            .or_else(|| params.get("service.vendor"))
            .or_else(|| params.get("os.vendor"))
            .or_else(|| params.get("hw.vendor"))
            .cloned();

        let product = params
            .get("cpe.product")
            .or_else(|| params.get("service.product"))
            .or_else(|| params.get("os.product"))
            .or_else(|| params.get("hw.product"))
            .cloned();

        let version = params
            .get("cpe.version")
            .or_else(|| params.get("service.version"))
            .or_else(|| params.get("os.version"))
            .or_else(|| params.get("hw.version"))
            .cloned();

        let update = params
            .get("cpe.update")
            .or_else(|| params.get("service.update"))
            .or_else(|| params.get("os.update"))
            .or_else(|| params.get("hw.update"))
            .cloned()
            .unwrap_or_else(|| "*".to_string());

        if let (Some(v), Some(p)) = (vendor, product) {
            let part = if params.contains_key("os.vendor") {
                "o"
            } else if params.contains_key("hw.vendor") {
                "h"
            } else {
                "a"
            };

            let ver = version.unwrap_or_else(|| "*".to_string());

            // Format: cpe:2.3:part:vendor:product:version:update:edition:language:sw_edition:target_sw:target_hw:other
            let cpe = format!(
                "cpe:2.3:{}:{}:{}:{}:{}:*:*:*:*:*:*",
                part,
                self.escape_cpe_field(&v),
                self.escape_cpe_field(&p),
                self.escape_cpe_field(&ver),
                self.escape_cpe_field(&update)
            );
            params.insert("cpe23".to_string(), cpe);
        }
    }

    /// Escape characters for CPE 2.3 as per specification
    fn escape_cpe_field(&self, field: &str) -> String {
        if field == "*" {
            return "*".to_string();
        }
        let mut escaped = String::with_capacity(field.len());
        for c in field.chars() {
            if c.is_alphanumeric() || c == '_' || c == '~' {
                escaped.push(c);
            } else if c == ' ' || c == '-' || c == '.' {
                // These are allowed in some contexts but often replaced or escaped
                escaped.push(c);
            } else {
                escaped.push('\\');
                escaped.push(c);
            }
        }
        // Replace spaces with underscores which is common in CPEs
        escaped.replace(' ', "_")
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

    #[test]
    fn test_cpe_generation_service() {
        let interpolator = ParamInterpolator::new();
        let mut params = HashMap::new();
        params.insert("service.vendor".to_string(), "Apache".to_string());
        params.insert("service.product".to_string(), "HTTP Server".to_string());
        params.insert("service.version".to_string(), "2.4.41".to_string());

        interpolator.process_cpe_params(&mut params);

        assert_eq!(
            params.get("cpe23"),
            Some(&"cpe:2.3:a:Apache:HTTP_Server:2.4.41:*:*:*:*:*:*:*".to_string())
        );
    }

    #[test]
    fn test_cpe_generation_os() {
        let interpolator = ParamInterpolator::new();
        let mut params = HashMap::new();
        params.insert("os.vendor".to_string(), "Microsoft".to_string());
        params.insert("os.product".to_string(), "Windows".to_string());
        params.insert("os.version".to_string(), "10".to_string());
        params.insert("os.update".to_string(), "20H2".to_string());

        interpolator.process_cpe_params(&mut params);

        assert_eq!(
            params.get("cpe23"),
            Some(&"cpe:2.3:o:Microsoft:Windows:10:20H2:*:*:*:*:*:*".to_string())
        );
    }

    #[test]
    fn test_cpe_escaping() {
        let interpolator = ParamInterpolator::new();
        let mut params = HashMap::new();
        params.insert(
            "service.vendor".to_string(),
            "Vendor With Space".to_string(),
        );
        params.insert(
            "service.product".to_string(),
            "Product(Special)".to_string(),
        );
        params.insert("service.version".to_string(), "1.0".to_string());

        interpolator.process_cpe_params(&mut params);

        // Space becomes underscore
        // Parentheses should be escaped
        assert_eq!(
            params.get("cpe23"),
            Some(&"cpe:2.3:a:Vendor_With_Space:Product\\(Special\\):1.0:*:*:*:*:*:*:*".to_string())
        );
    }
}
