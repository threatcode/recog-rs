// src/bin/fingerprint_analytics.rs
use recog::analytics;
use recog::loader::load_fingerprints_from_file;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: fingerprint_analytics <fingerprint_xml_file>");
        std::process::exit(1);
    }
    let path = &args[1];
    match load_fingerprints_from_file(path) {
        Ok(db) => analytics::print_overlaps(&db),
        Err(e) => {
            eprintln!("Failed to load fingerprints: {}", e);
            std::process::exit(1);
        }
    }
}
