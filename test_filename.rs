use recog::{load_fingerprints_from_xml, Matcher};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test filename-based example loading
    let xml_content = r#"
        <fingerprints>
            <fingerprint pattern="Apache/.*">
                <description>Apache HTTP Server</description>
                <example filename="examples/apache_banner.txt">
                    <param name="hw.product" value="Apache"/>
                </example>
            </fingerprint>
        </fingerprints>
    "#;

    println!("Testing filename-based example loading...");

    // Load the fingerprint database
    let db = load_fingerprints_from_xml(xml_content)?;
    println!("Loaded {} fingerprints", db.fingerprints.len());

    // Check the example was loaded correctly
    let fp = &db.fingerprints[0];
    println!("Fingerprint: {}", fp.description);
    println!("Examples: {}", fp.examples.len());

    if let Some(example) = fp.examples.get(0) {
        println!("Example value: {}", example.value);
        println!("Example is_base64: {}", example.is_base64);
        println!("Expected params: {:?}", example.expected_values);
    }

    // Test matching
    let matcher = Matcher::new(db);
    let test_banner = "Apache/2.4.41 (Ubuntu) Server Header";
    let results = matcher.match_text(test_banner);

    println!("\nMatching against: {}", test_banner);
    println!("Found {} matches", results.len());

    for result in results {
        println!("  Description: {}", result.fingerprint.description);
        println!("  Params: {:?}", result.params);
    }

    println!("\nTest completed successfully!");
    Ok(())
}
