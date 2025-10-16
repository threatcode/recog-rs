//! Performance benchmarks for XML loading operations
//!
//! These benchmarks measure the performance of XML parsing and fingerprint database
//! construction, which are critical for application startup performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use recog::loader::load_fingerprints_from_xml;

/// Generate XML with varying numbers of fingerprints for scaling tests
fn generate_test_xml(count: usize) -> String {
    let mut xml = String::from(r#"<fingerprints matches="test" protocol="test" database_type="service" preference="0.90">"#);

    for i in 0..count {
        xml.push_str(&format!(r#"
            <fingerprint pattern="^TestPattern{}: (.+)$">
                <description>Test Pattern {}</description>
                <example>TestPattern{}: value{}</example>
                <param pos="1" name="value"/>
            </fingerprint>
        "#, i, i, i, i));
    }
    xml.push_str("</fingerprints>");
    xml
}

fn benchmark_xml_parsing_10(c: &mut Criterion) {
    let xml = generate_test_xml(10);
    c.bench_function("xml_parsing_10_fingerprints", |b| {
        b.iter(|| {
            black_box(load_fingerprints_from_xml(&xml).unwrap());
        })
    });
}

fn benchmark_xml_parsing_100(c: &mut Criterion) {
    let xml = generate_test_xml(100);
    c.bench_function("xml_parsing_100_fingerprints", |b| {
        b.iter(|| {
            black_box(load_fingerprints_from_xml(&xml).unwrap());
        })
    });
}

fn benchmark_xml_parsing_1000(c: &mut Criterion) {
    let xml = generate_test_xml(1000);
    c.bench_function("xml_parsing_1000_fingerprints", |b| {
        b.iter(|| {
            black_box(load_fingerprints_from_xml(&xml).unwrap());
        })
    });
}

fn benchmark_xml_parsing_5000(c: &mut Criterion) {
    let xml = generate_test_xml(5000);
    c.bench_function("xml_parsing_5000_fingerprints", |b| {
        b.iter(|| {
            black_box(load_fingerprints_from_xml(&xml).unwrap());
        })
    });
}

fn benchmark_xml_memory_usage(c: &mut Criterion) {
    // Test memory efficiency with a moderately large database
    let xml = generate_test_xml(1000);

    c.bench_function("xml_memory_usage", |b| {
        b.iter(|| {
            let db = black_box(load_fingerprints_from_xml(&xml).unwrap());
            black_box(db.fingerprints.len());
            // Explicit drop to ensure cleanup
            drop(db);
        })
    });
}

criterion_group!(
    benches,
    benchmark_xml_parsing_10,
    benchmark_xml_parsing_100,
    benchmark_xml_parsing_1000,
    benchmark_xml_parsing_5000,
    benchmark_xml_memory_usage
);
criterion_main!(benches);
