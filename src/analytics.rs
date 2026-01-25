// src/analytics.rs
use crate::fingerprint::FingerprintDatabase;
use std::collections::HashMap;

/// Extract a literal substring from a regex pattern (same heuristic as Matcher)
fn extract_literal(pattern: &str) -> Option<String> {
    pattern
        .split(|c: char| !c.is_alphanumeric() && c != ' ')
        .filter(|s| s.len() >= 4)
        .max_by_key(|s| s.len())
        .map(|s| s.to_string())
}

/// Find overlapping fingerprint patterns based on shared literals.
/// Returns a vector of (i, j) index pairs where i < j and the patterns share a literal.
pub fn find_overlaps(db: &FingerprintDatabase) -> Vec<(usize, usize)> {
    // Map literal -> list of fingerprint indices containing it
    let mut literal_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, fp) in db.fingerprints.iter().enumerate() {
        if let Some(lit) = extract_literal(fp.pattern.as_str()) {
            literal_map.entry(lit).or_default().push(idx);
        }
    }
    let mut overlaps = Vec::new();
    for indices in literal_map.values() {
        if indices.len() > 1 {
            for i in 0..indices.len() {
                for j in i + 1..indices.len() {
                    overlaps.push((indices[i], indices[j]));
                }
            }
        }
    }
    overlaps
}

/// Print overlapping fingerprint descriptions to stdout.
pub fn print_overlaps(db: &FingerprintDatabase) {
    let overlaps = find_overlaps(db);
    if overlaps.is_empty() {
        println!("No overlapping patterns detected.");
        return;
    }
    println!("Detected overlapping patterns:");
    for (i, j) in overlaps {
        let a = &db.fingerprints[i];
        let b = &db.fingerprints[j];
        println!(" - [{}] <-> [{}]", a.description, b.description);
    }
}
