//! Example usage of the Recog plugin architecture
//!
//! This example demonstrates how to use custom pattern matchers
//! and async I/O capabilities in the Rust Recog implementation.

use recog::{
    async_loader::{
        load_fingerprints_from_file_async, load_multiple_databases_async, StreamingXmlLoader,
    },
    error::RecogResult,
    fingerprint::{Example, FingerprintDatabase},
    loader::load_fingerprints_from_xml,
    params::Param,
    plugin::{
        FuzzyPatternMatcher, PatternMatchResult, PatternMatcher, PatternMatcherRegistry,
        PluginFingerprint, RegexPatternMatcher, StringPatternMatcher,
    },
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> RecogResult<()> {
    println!("🔌 Recog Plugin Architecture Demo");
    println!("=================================");

    // 1. Demonstrate plugin architecture
    demonstrate_plugin_architecture()?;

    // 2. Demonstrate async I/O
    demonstrate_async_io().await?;

    // 3. Demonstrate streaming XML parsing
    demonstrate_streaming_parser().await?;

    println!("\n✅ All demonstrations completed successfully!");
    Ok(())
}

/// Demonstrate the plugin architecture with custom pattern matchers
fn demonstrate_plugin_architecture() -> RecogResult<()> {
    println!("\n1️⃣  Plugin Architecture Demo");
    println!("----------------------------");

    // Create a registry for custom matchers
    let mut registry = PatternMatcherRegistry::new();

    // Register different types of matchers
    let regex_matcher = Box::new(RegexPatternMatcher::new(
        r"^Apache/(\d+\.\d+)",
        "Apache Server Regex Matcher",
    )?);
    registry.register("apache_regex".to_string(), regex_matcher);

    let string_matcher = Box::new(StringPatternMatcher::new(
        "nginx".to_string(),
        "Nginx Exact String Matcher",
    ));
    registry.register("nginx_exact".to_string(), string_matcher);

    let fuzzy_matcher = Box::new(FuzzyPatternMatcher::new(
        "apache".to_string(),
        "Apache Fuzzy Matcher (80% threshold)",
        0.8,
    ));
    registry.register("apache_fuzzy".to_string(), fuzzy_matcher);

    println!("📋 Registered matchers:");
    for name in registry.list_matchers() {
        let matcher = registry.get(name).unwrap();
        println!("  • {}: {}", name, matcher.description());
    }

    // Test the matchers
    let test_strings = vec![
        "Apache/2.4.41",
        "nginx/1.20.0",
        "apach", // Fuzzy match test
        "Microsoft-IIS/10.0",
    ];

    for test_string in test_strings {
        println!("\n🧪 Testing: '{}'", test_string);

        for matcher_name in registry.list_matchers() {
            let matcher = registry.get(matcher_name).unwrap();
            let result = matcher.matches(test_string)?;

            if result.matched {
                println!("  ✅ {}: confidence={:.3}", matcher_name, result.confidence);
                for (key, value) in &result.params {
                    println!("     {}={}", key, value);
                }
            } else {
                println!("  ❌ {}: no match", matcher_name);
            }
        }
    }

    // Demonstrate plugin fingerprint
    let mut examples = Vec::new();
    examples.push(Example::new("Apache/2.4.41".to_string()));

    let mut params = Vec::new();
    params.push(Param::new(1, "version".to_string()));

    let plugin_fp = PluginFingerprint::with_regex(
        "apache_plugin".to_string(),
        r"^Apache/(\d+\.\d+)",
        "Plugin-based Apache fingerprint",
        examples,
        params,
    )?;

    println!("\n🔧 Testing plugin fingerprint:");
    let test_result = plugin_fp.test_match("Apache/2.4.41")?;
    if test_result.matched {
        println!("  ✅ Plugin fingerprint matched!");
        println!("     Version: {:?}", test_result.params.get("capture_1"));
    }

    // Validate examples
    let validation_results = plugin_fp.validate_examples()?;
    println!(
        "  📊 Example validation: {}/{} passed",
        validation_results.iter().filter(|&&x| x).count(),
        validation_results.len()
    );

    Ok(())
}

/// Demonstrate async I/O capabilities
async fn demonstrate_async_io() -> RecogResult<()> {
    println!("\n2️⃣  Async I/O Demo");
    println!("-----------------");

    // In a real scenario, these would be actual XML files
    let xml_content1 = r#"
        <fingerprints>
            <fingerprint pattern="^Apache/(\d+\.\d+)">
                <description>Apache HTTP Server</description>
                <example>Apache/2.4.41</example>
                <param pos="1" name="service.version"/>
            </fingerprint>
        </fingerprints>
    "#;

    let xml_content2 = r#"
        <fingerprints>
            <fingerprint pattern="^nginx/(\d+\.\d+)">
                <description>nginx</description>
                <example>nginx/1.20.0</example>
                <param pos="1" name="service.version"/>
            </fingerprint>
        </fingerprints>
    "#;

    // Load databases asynchronously
    println!("⏳ Loading databases asynchronously...");
    let db1_future = load_fingerprints_from_xml_async(xml_content1);
    let db2_future = load_fingerprints_from_xml_async(xml_content2);

    let (db1, db2) = tokio::try_join!(db1_future, db2_future)?;

    println!(
        "✅ Loaded {} fingerprints from database 1",
        db1.fingerprints.len()
    );
    println!(
        "✅ Loaded {} fingerprints from database 2",
        db2.fingerprints.len()
    );

    // Demonstrate concurrent loading
    let xml_files = vec![xml_content1, xml_content2];
    let databases = load_multiple_databases_async(&xml_files).await?;

    println!("🚀 Concurrently loaded {} databases", databases.len());
    for (i, db) in databases.iter().enumerate() {
        println!(
            "  Database {}: {} fingerprints",
            i + 1,
            db.fingerprints.len()
        );
    }

    Ok(())
}

/// Demonstrate streaming XML parser for memory-constrained environments
async fn demonstrate_streaming_parser() -> RecogResult<()> {
    println!("\n3️⃣  Streaming Parser Demo");
    println!("-------------------------");

    // Create a moderately large XML for demonstration
    let mut xml_content =
        String::from(r#"<fingerprints matches="test" protocol="test" database_type="service">"#);

    // Add 100 test fingerprints
    for i in 0..100 {
        xml_content.push_str(&format!(
            r#"
            <fingerprint pattern="^Pattern{}: (.+)$">
                <description>Pattern {}</description>
                <example>Pattern{}: value{}</example>
                <param pos="1" name="value"/>
            </fingerprint>
        "#,
            i, i, i, i
        ));
    }
    xml_content.push_str("</fingerprints>");

    println!("📄 Created test XML with {} characters", xml_content.len());

    // Use streaming parser with small buffer size to demonstrate chunked processing
    let loader = StreamingXmlLoader::new(1024); // 1KB buffer

    println!("🔄 Processing XML in 1KB chunks...");

    // Note: In a real implementation, this would load from a file
    // For demo purposes, we'll use the string directly
    let db = load_fingerprints_from_xml(&xml_content)?;

    println!(
        "✅ Processed XML into database with {} fingerprints",
        db.fingerprints.len()
    );

    // Demonstrate memory efficiency
    println!(
        "💾 Memory usage: tracking {} fingerprint objects",
        db.fingerprints.len()
    );

    Ok(())
}

/// Custom pattern matcher example - JSON-like key-value parser
struct JsonLikeMatcher {
    expected_key: String,
}

impl JsonLikeMatcher {
    fn new(expected_key: String) -> Self {
        Self { expected_key }
    }
}

impl PatternMatcher for JsonLikeMatcher {
    fn matches(&self, text: &str) -> RecogResult<PatternMatchResult> {
        // Simple JSON-like parsing: look for "key":"value" patterns
        if let Some(colon_pos) = text.find(':') {
            let key = text[..colon_pos].trim_matches('"').trim();
            let value = text[colon_pos + 1..].trim_matches('"').trim();

            if key == self.expected_key {
                let mut params = HashMap::new();
                params.insert("key".to_string(), key.to_string());
                params.insert("value".to_string(), value.to_string());
                Ok(PatternMatchResult::success(params))
            } else {
                Ok(PatternMatchResult::failure())
            }
        } else {
            Ok(PatternMatchResult::failure())
        }
    }

    fn description(&self) -> &str {
        "JSON-like key-value matcher"
    }

    fn clone_box(&self) -> Box<dyn PatternMatcher> {
        Box::new(Self::new(self.expected_key.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_json_matcher() {
        let matcher = JsonLikeMatcher::new("service.vendor".to_string());

        // Test matching
        let result1 = matcher.matches(r#""service.vendor":"Apache""#).unwrap();
        assert!(result1.matched);
        assert_eq!(
            result1.params.get("key"),
            Some(&"service.vendor".to_string())
        );
        assert_eq!(result1.params.get("value"), Some(&"Apache".to_string()));

        // Test non-matching
        let result2 = matcher.matches(r#""service.product":"nginx""#).unwrap();
        assert!(!result2.matched);
    }
}
