use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use recog::{load_fingerprints_from_file, Matcher};
use std::io::{self};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "recog_match")]
#[command(about = "Match input text against Recog fingerprints")]
struct Args {
    /// Fingerprint database file
    #[arg(short, long)]
    db: PathBuf,

    /// Input file (stdin if not provided)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Base64 decode input before matching
    #[arg(short, long)]
    base64: bool,

    /// Output format (json, text)
    #[arg(short, long, default_value = "json")]
    format: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load fingerprint database
    let db = load_fingerprints_from_file(&args.db)?;
    let matcher = Matcher::new(db);

    // Read input
    let input_text = if let Some(input_path) = args.input {
        std::fs::read_to_string(input_path)?
    } else {
        let stdin = io::stdin();
        let mut lines = stdin.lines();
        let mut content = String::new();

        while let Some(Ok(line)) = lines.next() {
            content.push_str(&line);
            content.push('\n');
        }

        content.trim().to_string()
    };

    let text = if args.base64 {
        let decoded = general_purpose::STANDARD.decode(&input_text)?;
        String::from_utf8(decoded)?
    } else {
        input_text
    };

    // Perform matching
    let results = matcher.match_text(&text);

    // Output results
    match args.format.as_str() {
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
            eprintln!("Unknown output format: {}", args.format);
            std::process::exit(1);
        }
    }

    Ok(())
}
