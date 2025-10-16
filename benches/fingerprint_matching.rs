//! Performance benchmarks for fingerprint matching operations
//!
//! These benchmarks quantify the performance characteristics of the Rust Recog implementation
//! compared to other language implementations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use recog::{
    fingerprint::{Fingerprint, FingerprintDatabase},
    loader::load_fingerprints_from_xml,
    matcher::Matcher,
};
use std::collections::HashMap;

/// Create a test fingerprint database for benchmarking
fn create_test_database() -> FingerprintDatabase {
    let xml = r#"
        <fingerprints>
            <fingerprint pattern="^Apache/(\d+\.\d+)">
                <description>Apache HTTP Server</description>
                <example>Apache/2.4.41</example>
                <param pos="1" name="service.version"/>
            </fingerprint>
            <fingerprint pattern="^nginx/(\d+\.\d+)">
                <description>nginx</description>
                <example>nginx/1.20.0</example>
                <param pos="1" name="service.version"/>
            </fingerprint>
            <fingerprint pattern="^Microsoft-IIS/(\d+\.\d+)">
                <description>Microsoft IIS</description>
                <example>Microsoft-IIS/10.0</example>
                <param pos="1" name="service.version"/>
            </fingerprint>
        </fingerprints>
    "#;

    load_fingerprints_from_xml(xml).unwrap()
}

/// Create a larger test database for more comprehensive benchmarking
fn create_large_database() -> FingerprintDatabase {
    let mut xml = String::from("<fingerprints>");

    // Create 1000 fingerprints for comprehensive testing
    for i in 0..1000 {
        xml.push_str(&format!(
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
    xml.push_str("</fingerprints>");

    load_fingerprints_from_xml(&xml).unwrap()
}

fn benchmark_fingerprint_creation(c: &mut Criterion) {
    c.bench_function("fingerprint_creation", |b| {
        b.iter(|| {
            black_box(Fingerprint::new(r"^TestPattern/(\d+\.\d+)", "Test Pattern").unwrap());
        })
    });
}

fn benchmark_xml_loading_small(c: &mut Criterion) {
    let xml = r#"
        <fingerprints>
            <fingerprint pattern="^Apache/(\d+\.\d+)">
                <description>Apache HTTP Server</description>
                <example>Apache/2.4.41</example>
                <param pos="1" name="service.version"/>
            </fingerprint>
        </fingerprints>
    "#;

    c.bench_function("xml_loading_small", |b| {
        b.iter(|| {
            black_box(load_fingerprints_from_xml(xml).unwrap());
        })
    });
}

fn benchmark_xml_loading_large(c: &mut Criterion) {
    let mut xml = String::from("<fingerprints>");
    for i in 0..500 {
        xml.push_str(&format!(
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
    xml.push_str("</fingerprints>");

    c.bench_function("xml_loading_large", |b| {
        b.iter(|| {
            black_box(load_fingerprints_from_xml(&xml).unwrap());
        })
    });
}

fn benchmark_matcher_creation(c: &mut Criterion) {
    let db = create_test_database();

    c.bench_function("matcher_creation", |b| {
        b.iter(|| {
            black_box(Matcher::new(db.clone()));
        })
    });
}

fn benchmark_simple_matching(c: &mut Criterion) {
    let db = create_test_database();
    let matcher = Matcher::new(db);

    c.bench_function("simple_matching", |b| {
        b.iter(|| {
            black_box(matcher.match_text("Apache/2.4.41"));
        })
    });
}

fn benchmark_complex_matching(c: &mut Criterion) {
    let db = create_large_database();
    let matcher = Matcher::new(db);

    c.bench_function("complex_matching", |b| {
        b.iter(|| {
            black_box(matcher.match_text("Pattern500: value500"));
        })
    });
}

fn benchmark_batch_matching(c: &mut Criterion) {
    let db = create_test_database();
    let matcher = Matcher::new(db);

    let test_strings = vec![
        "Apache/2.4.41".to_string(),
        "nginx/1.20.0".to_string(),
        "Microsoft-IIS/10.0".to_string(),
        "Apache/2.2.22".to_string(),
        "nginx/1.18.0".to_string(),
    ];

    c.bench_function("batch_matching", |b| {
        b.iter(|| {
            black_box(matcher.match_batch(&test_strings));
        })
    });
}

fn benchmark_parameter_interpolation(c: &mut Criterion) {
    use recog::params::ParamInterpolator;

    let mut interpolator = ParamInterpolator::new();
    let mut params = HashMap::new();
    params.insert("service.vendor".to_string(), "Apache".to_string());
    params.insert("service.product".to_string(), "HTTP Server".to_string());
    params.insert("service.version".to_string(), "2.4.41".to_string());

    let template = "cpe:/a:{service.vendor}:{service.product}:{service.version}";

    c.bench_function("parameter_interpolation", |b| {
        b.iter(|| {
            black_box(interpolator.interpolate(template, &params));
        })
    });
}

fn benchmark_regex_compilation(c: &mut Criterion) {
    c.bench_function("regex_compilation", |b| {
        b.iter(|| {
            black_box(regex::Regex::new(r"^Apache/(\d+\.\d+)").unwrap());
        })
    });
}

criterion_group!(
    benches,
    benchmark_fingerprint_creation,
    benchmark_xml_loading_small,
    benchmark_xml_loading_large,
    benchmark_matcher_creation,
    benchmark_simple_matching,
    benchmark_complex_matching,
    benchmark_batch_matching,
    benchmark_parameter_interpolation,
    benchmark_regex_compilation
);
criterion_main!(benches);
