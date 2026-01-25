// Example: Using the fingerprint analytics tool

use recog::analytics;
use recog::loader::load_fingerprints_from_xml;

fn main() {
    // Sample XML with intentionally overlapping patterns
    let xml = r#"
        <fingerprints>
            <fingerprint pattern="^Apache/([\d.]+)" description="Apache HTTP Server">
                <param pos="1" name="version"/>
            </fingerprint>
            <fingerprint pattern="^Apache/([\d.]+) \((.+)\)" description="Apache with OS">
                <param pos="1" name="version"/>
                <param pos="2" name="os"/>
            </fingerprint>
            <fingerprint pattern="^nginx/([\d.]+)" description="Nginx">
                <param pos="1" name="version"/>
            </fingerprint>
            <fingerprint pattern="^nginx/([\d.]+) on (.+)" description="Nginx with platform">
                <param pos="1" name="version"/>
                <param pos="2" name="platform"/>
            </fingerprint>
            <fingerprint pattern="^Microsoft-IIS/([\d.]+)" description="Microsoft IIS">
                <param pos="1" name="version"/>
            </fingerprint>
        </fingerprints>
    "#;

    match load_fingerprints_from_xml(xml) {
        Ok(db) => {
            println!("Loaded {} fingerprints\n", db.fingerprints.len());

            // Find overlapping patterns
            let overlaps = analytics::find_overlaps(&db);

            println!("Analysis Results:");
            println!("================\n");

            if overlaps.is_empty() {
                println!("✓ No overlapping patterns detected.");
            } else {
                println!("⚠ Found {} overlapping pattern pairs:\n", overlaps.len());

                for (i, j) in &overlaps {
                    let fp1 = &db.fingerprints[*i];
                    let fp2 = &db.fingerprints[*j];

                    println!("Overlap detected:");
                    println!("  [{}] {}", i, fp1.description);
                    println!("      Pattern: {}", fp1.pattern);
                    println!("  [{}] {}", j, fp2.description);
                    println!("      Pattern: {}", fp2.pattern);
                    println!();
                }

                println!("Recommendation: Review these patterns to ensure they're intentional.");
                println!("Overlapping patterns may cause:");
                println!("  - Redundant matches");
                println!("  - Performance degradation");
                println!("  - Ambiguous results");
            }
        }
        Err(e) => {
            eprintln!("Failed to load fingerprints: {}", e);
        }
    }
}
