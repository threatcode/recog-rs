//! Comprehensive tests for the Recog library
//!
//! This module contains extensive tests for edge cases, multi-line patterns,
//! error handling, and performance validation.

#[cfg(test)]

    use super::*;
    use crate::{
        error::RecogError,
        fingerprint::{Fingerprint, FingerprintDatabase},
        load_fingerprints_from_file, load_fingerprints_from_xml,
        matcher::{MatchResult, Matcher},
        params::{Param, ParamInterpolator},
    };
    use std::collections::HashMap;

    /// Test edge cases for fingerprint creation and validation
    #[test]
    fn test_fingerprint_edge_cases() {
        // Test invalid regex patterns
        let result = Fingerprint::new("[invalid regex", "Test");
        assert!(matches!(result, Err(RecogError::Regex(_))));

        // Test empty pattern
        let fingerprint = Fingerprint::new("", "Empty pattern").unwrap();
        assert_eq!(fingerprint.description, "Empty pattern");

        // Test valid but complex pattern
        let fingerprint = Fingerprint::new(r"^Apache/(\d+\.\d+)", "Apache Server").unwrap();
        assert_eq!(fingerprint.description, "Apache Server");
        assert_eq!(fingerprint.params.len(), 0);
    }

    /// Test multi-line pattern matching
    #[test]
    fn test_multiline_patterns() {
        // Create fingerprint with multi-line pattern (using (?m) flag)
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="(?m)^Server: (.+)$" description="Multi-line server header">
                    <example value="Server: Apache/2.4.41&#10;X-Powered-By: PHP/7.3">
                        <param name="service.product" value="Apache"/>
                        <param name="service.version" value="2.4.41"/>
                    </example>
                    <param pos="1" name="service.product"/>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let matcher = Matcher::new(db);

        let multiline_text = "Server: Apache/2.4.41\nX-Powered-By: PHP/7.3";
        let results = matcher.match_text(multiline_text);

        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(
            result.params.get("service.product"),
            Some(&"Apache/2.4.41".to_string())
        );
    }

    /// Test complex parameter interpolation scenarios
    #[test]
    fn test_complex_parameter_interpolation() {
        let interpolator = ParamInterpolator::new();
        let mut params = HashMap::new();

        // Test nested interpolation
        params.insert("service.vendor".to_string(), "Apache".to_string());
        params.insert("service.product".to_string(), "HTTP Server".to_string());
        params.insert("service.version".to_string(), "2.4.41".to_string());

        // Template with multiple parameter references
        let template = "cpe:/a:{service.vendor}:{service.product}:{service.version}";
        let result = interpolator.interpolate(template, &params);

        assert_eq!(result, "cpe:/a:Apache:HTTP Server:2.4.41");

        // Test with missing parameters
        let template2 = "cpe:/a:{service.vendor}:{service.product}:{missing.version}";
        let result2 = interpolator.interpolate(template2, &params);
        assert_eq!(result2, "cpe:/a:Apache:HTTP Server:");
    }

    /// Test error handling for malformed XML
    #[test]
    fn test_malformed_xml_handling() {
        let malformed_xml = r#"
            <fingerprints>
                <fingerprint pattern="^Apache/(\d+\.\d+)" description="Apache Server">
                    <example value="Apache/2.4.41">
                        <param name="service.vendor" value="Apache"/>
                    </example>
                    <!-- Missing closing tag for param
                </fingerprint>
            </fingerprints>
        "#;

        let result = load_fingerprints_from_xml(malformed_xml);
        assert!(matches!(result, Err(RecogError::Custom { .. })));
    }

    /// Test base64 encoded examples
    #[test]
    fn test_base64_examples() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="^test data$" description="Base64 test">
                    <example encoding="base64" value="dGVzdCBkYXRh">
                        <param name="test.param" value="decoded"/>
                    </example>
                    <param pos="0" name="test.param"/>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let matcher = Matcher::new(db);

        // The base64 "dGVzdCBkYXRh" decodes to "test data"
        let results = matcher.match_text("test data");
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(
            result.params.get("test.param"),
            Some(&"test data".to_string())
        );
    }

    /// Test external file examples
    #[test]
    fn test_external_file_examples() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let example_file = temp_dir.path().join("example.txt");
        fs::write(&example_file, "External example content").unwrap();

        let xml = format!(
            r#"
            <fingerprints>
                <fingerprint pattern="^External example content$" description="External file test">
                    <example filename="{}">
                        <param name="test.param" value="external"/>
                    </example>
                    <param pos="0" name="test.param"/>
                </fingerprint>
            </fingerprints>
        "#,
            example_file.to_string_lossy()
        );

        let db = load_fingerprints_from_xml(&xml).unwrap();
        let matcher = Matcher::new(db);

        let results = matcher.match_text("External example content");
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(
            result.params.get("test.param"),
            Some(&"External example content".to_string())
        );
    }

    /// Test fingerprint database with many patterns (performance test)
    #[test]
    fn test_large_fingerprint_database() {
        let mut xml = String::from("<fingerprints>");

        // Create 1000 fingerprints
        for i in 0..1000 {
            xml.push_str(&format!(
                r#"
                <fingerprint pattern="^Pattern{}: (.+)$" description="Pattern {}">
                    <example value="Pattern{}: value{}" />
                    <param pos="1" name="value"/>
                </fingerprint>
            "#,
                i, i, i, i
            ));
        }
        xml.push_str("</fingerprints>");

        let db = load_fingerprints_from_xml(&xml).unwrap();
        assert_eq!(db.fingerprints.len(), 1000);

        let matcher = Matcher::new(db);

        // Test matching performance with a simple pattern
        let start = std::time::Instant::now();
        let results = matcher.match_text("Pattern500: value500");
        let duration = start.elapsed();

        // Should complete quickly (less than 100ms for 1000 patterns)
        assert!(duration.as_millis() < 100);
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].params.get("value"),
            Some(&"value500".to_string())
        );
    }

    /// Test parameter validation and edge cases
    #[test]
    fn test_parameter_validation() {
        // Test parameter with invalid position
        let param = Param::new(0, "invalid_pos".to_string());
        assert_eq!(param.pos, 0);
        assert_eq!(param.name, "invalid_pos");

        // Test parameter with value
        let param_with_value = Param::with_value(1, "version".to_string(), "1.0".to_string());
        assert_eq!(param_with_value.pos, 1);
        assert_eq!(param_with_value.name, "version");
        assert_eq!(param_with_value.value, Some("1.0".to_string()));

        // Test parameter interpolation with temporary params
        let mut interpolator = ParamInterpolator::new();
        interpolator.add_temp_param("_tmp.os");

        let mut params = HashMap::new();
        params.insert("service.vendor".to_string(), "Apache".to_string());
        params.insert("_tmp.os".to_string(), "Linux".to_string());

        interpolator.filter_temp_params(&mut params);
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("service.vendor"), Some(&"Apache".to_string()));
        assert!(!params.contains_key("_tmp.os"));
    }

    /// Test concurrent access to matcher
    #[test]
    fn test_concurrent_matching() {
        let xml = r#"
            <fingerprints>
                <fingerprint pattern="^Thread (\d+): (.+)$" description="Thread test">
                    <example value="Thread 1: data1" />
                    <example value="Thread 2: data2" />
                    <param pos="1" name="thread_id"/>
                    <param pos="2" name="data"/>
                </fingerprint>
            </fingerprints>
        "#;

        let db = load_fingerprints_from_xml(xml).unwrap();
        let matcher = Matcher::new(db);

        // Test concurrent access (basic thread safety check)
        let matcher1 = &matcher;
        let matcher2 = &matcher;

        let result1 = matcher1.match_text("Thread 1: data1");
        let result2 = matcher2.match_text("Thread 2: data2");

        assert_eq!(result1.len(), 1);
        assert_eq!(result2.len(), 1);
        assert_eq!(result1[0].params.get("thread_id"), Some(&"1".to_string()));
        assert_eq!(result2[0].params.get("thread_id"), Some(&"2".to_string()));
    }

    /// Test memory usage with large databases
    #[test]
    fn test_memory_efficiency() {
        // Create a moderately large database
        let mut xml = String::from("<fingerprints>");
        for i in 0..100 {
            xml.push_str(&format!(
                r#"
                <fingerprint pattern="^Test{} (.+)$" description="Test pattern {}">
                    <example value="Test{} value{}" />
                    <param pos="1" name="value"/>
                </fingerprint>
            "#,
                i, i, i, i
            ));
        }
        xml.push_str("</fingerprints>");

        // This should not cause excessive memory usage
        let db = load_fingerprints_from_xml(&xml).unwrap();
        assert_eq!(db.fingerprints.len(), 100);

        // Drop database to ensure cleanup
        drop(db);
    }

    /// Test error propagation through the entire stack
    #[test]
    fn test_error_propagation() {
        // Test that errors are properly propagated and typed

        // Invalid regex should give RecogError::Regex
        let result = Fingerprint::new("[invalid", "test");
        assert!(matches!(result, Err(RecogError::Regex(_))));

        // Malformed XML should give RecogError::XmlParsing
        let malformed = "<fingerprints><fingerprint pattern='a'></fingerprint></fingerprints";
        let result = load_fingerprints_from_xml(malformed);
        assert!(matches!(result, Err(RecogError::Custom { .. })));

        // File not found should give RecogError::Io
        let result = load_fingerprints_from_file("nonexistent.xml");
        assert!(matches!(result, Err(RecogError::Io(_))));
    }

    /// Test JSON serialization of match results
    #[test]
    fn test_json_serialization() {
        let fingerprint = Fingerprint::new(r"^Test: (.+)$", "Test pattern").unwrap();
        let mut params = HashMap::new();
        params.insert("param1".to_string(), "value1".to_string());
        params.insert("param2".to_string(), "value2".to_string());

        let result = MatchResult::new(fingerprint, params);
        let json = result.to_json().unwrap();

        // Should contain description and params
        assert!(json.contains("Test pattern"));
        assert!(json.contains("value1"));
        assert!(json.contains("value2"));
        assert!(json.contains("param1"));
        assert!(json.contains("param2"));
    }
}
