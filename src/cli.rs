use crate::{
    error::{RecogError, RecogResult},
    load_fingerprints_from_file, Matcher,
};
use clap::{Parser, Subcommand};
use std::io::{self, Read};
use std::path::PathBuf;

/// Recog CLI tool for fingerprint verification and matching
#[derive(Parser)]
#[command(name = "recog")]
#[command(about = "Fingerprint-based recognition tool")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Match input against fingerprints
    Match {
        /// Input text or file to match
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Fingerprint database file
        #[arg(short, long)]
        db: PathBuf,

        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Base64 decode input before matching
        #[arg(short, long)]
        base64: bool,
    },
    /// Verify fingerprint coverage against examples
    Verify {
        /// Fingerprint database file
        #[arg(short, long)]
        db: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show detailed results
        #[arg(short, long)]
        verbose: bool,
    },
}

/// Run the CLI application
pub fn run() -> RecogResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Match {
            input,
            db,
            format,
            base64,
        } => run_match(input, db, format, base64),
        Commands::Verify {
            db,
            format,
            verbose,
        } => run_verify(db, format, verbose),
    }
}

fn run_match(
    input: Option<PathBuf>,
    db_path: PathBuf,
    format: String,
    base64: bool,
) -> RecogResult<()> {
    // Load fingerprint database
    let db = load_fingerprints_from_file(&db_path)?;

    // Read input text
    let input_text = if let Some(input_path) = input {
        std::fs::read_to_string(input_path)?
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer.trim().to_string()
    };

    let text = if base64 {
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &input_text)?;
        String::from_utf8(decoded)?
    } else {
        input_text
    };

    // Perform matching
    let matcher = Matcher::new(db);
    let results = matcher.match_text(&text);

    // Output results
    match format.as_str() {
        "json" => {
            for result in results {
                println!("{}", result.to_json()?);
            }
        }
        "text" => {
            for result in results {
                println!("Description: {}", result.fingerprint.description);
                for (key, value) in result.params {
                    println!("  {}: {}", key, value);
                }
                println!();
            }
        }
        _ => {
            eprintln!("Unknown output format: {}", format);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_verify(db_path: PathBuf, format: String, verbose: bool) -> RecogResult<()> {
    // Load fingerprint database
    let db = load_fingerprints_from_file(&db_path)?;

    let mut total_examples = 0;
    let mut matched_examples = 0;

    for fingerprint in &db.fingerprints {
        for example in &fingerprint.examples {
            total_examples += 1;

            let text = if example.is_base64 {
                let decoded = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &example.value,
                )?;
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
            }

            if verbose {
                if matched {
                    println!("✓ {}", fingerprint.description);
                } else {
                    println!("✗ {} (no match for: {})", fingerprint.description, text);
                }
            }
        }
    }

    match format.as_str() {
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
            if total_examples > 0 {
                println!(
                    "  Success rate: {:.2}%",
                    (matched_examples as f64 / total_examples as f64) * 100.0
                );
            }
        }
        _ => {
            eprintln!("Unknown output format: {}", format);
            std::process::exit(1);
        }
    }

    Ok(())
}
