use clap::Parser;
use recog::{load_fingerprints_from_file, Matcher};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "recog_verify")]
#[command(about = "Verify fingerprint coverage against examples")]
struct Args {
    /// Fingerprint database file
    #[arg(short, long)]
    db: PathBuf,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Show detailed results for each example
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load fingerprint database
    let db = load_fingerprints_from_file(&args.db)?;

    let mut total_examples = 0;
    let mut matched_examples = 0;
    let mut failures = Vec::new();

    for fingerprint in &db.fingerprints {
        for example in &fingerprint.examples {
            total_examples += 1;

            let text = if example.is_base64 {
                let decoded = base64::decode(&example.value)?;
                String::from_utf8(decoded)?
            } else {
                example.value.clone()
            };

            let matcher = Matcher::new(db.clone());
            let results = matcher.match_text(&text);

            let matched = results
                .iter()
                .any(|r| r.fingerprint.description == fingerprint.description);

            if matched {
                matched_examples += 1;
            } else {
                failures.push((fingerprint.description.clone(), text.clone()));
            }

            if args.verbose {
                if matched {
                    println!("✓ {} -> {}", fingerprint.description, text);
                } else {
                    println!("✗ {} -> {}", fingerprint.description, text);
                }
            }
        }
    }

    // Output results
    match args.format.as_str() {
        "json" => {
            let mut result = serde_json::Map::new();
            result.insert(
                "total_examples".to_string(),
                serde_json::Value::Number(total_examples.into()),
            );
            result.insert(
                "matched_examples".to_string(),
                serde_json::Value::Number(matched_examples.into()),
            );
            result.insert(
                "failed_examples".to_string(),
                serde_json::Value::Number(failures.len().into()),
            );

            if args.verbose {
                let failures_json: Vec<serde_json::Value> = failures
                    .iter()
                    .map(|(desc, text)| {
                        let mut obj = serde_json::Map::new();
                        obj.insert(
                            "description".to_string(),
                            serde_json::Value::String(desc.clone()),
                        );
                        obj.insert("input".to_string(), serde_json::Value::String(text.clone()));
                        serde_json::Value::Object(obj)
                    })
                    .collect();

                result.insert(
                    "failures".to_string(),
                    serde_json::Value::Array(failures_json),
                );
            }

            result.insert(
                "success_rate".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(if total_examples > 0 {
                        matched_examples as f64 / total_examples as f64
                    } else {
                        0.0
                    })
                    .unwrap_or(serde_json::Number::from(0)),
                ),
            );

            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "text" => {
            println!("Verification Results:");
            println!("  Total examples: {}", total_examples);
            println!("  Matched examples: {}", matched_examples);
            println!("  Failed examples: {}", failures.len());

            if total_examples > 0 {
                println!(
                    "  Success rate: {:.2}%",
                    (matched_examples as f64 / total_examples as f64) * 100.0
                );
            }

            if !failures.is_empty() && args.verbose {
                println!("\nFailures:");
                for (desc, text) in failures {
                    println!("  ✗ {} -> {}", desc, text);
                }
            }
        }
        _ => {
            eprintln!("Unknown output format: {}", args.format);
            std::process::exit(1);
        }
    }

    Ok(())
}
