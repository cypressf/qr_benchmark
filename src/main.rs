use anyhow::Result;
use clap::Parser;
use indicatif::ProgressBar;
use qr_benchmark::{benchmark, data, decoders};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;

const DATA_DIRS: &[&str] = &["./qrcodes/decoding", "./qrcodes/detection"];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of libraries to benchmark. If empty, all libraries are run.
    #[arg(short, long)]
    libs: Vec<String>,

    /// Number of iterations to run per image.
    #[arg(short = 'n', long, default_value_t = 5)]
    iterations: u32,

    /// List of categories to benchmark (e.g., 'blurred', 'glare'). If empty, all categories are run.
    #[arg(short, long)]
    categories: Vec<String>,

    /// Output CSV file path.
    #[arg(short, long, default_value = "raw_measurements.csv")]
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Data Discovery
    println!("Discovering test data from {:?}...", DATA_DIRS);
    let all_pairs = data::discover_test_data(DATA_DIRS, None)?;

    // Filter pairs by category if specified
    let pairs: Vec<_> = if args.categories.is_empty() {
        all_pairs
    } else {
        all_pairs
            .into_iter()
            .filter(|pair| args.categories.contains(&pair.category))
            .collect()
    };

    if pairs.is_empty() {
        eprintln!("No test data found for the specified categories.");
        if !args.categories.is_empty() {
            eprintln!("Requested categories: {:?}", args.categories);
        }
        return Ok(());
    }

    println!("Found {} test pairs.", pairs.len());

    if !args.categories.is_empty() {
        println!("Categories: {:?}", args.categories);
    }

    // 2. Setup Decoders
    let mut all_decoders: Vec<Box<dyn decoders::QrDecoder>> = Vec::new();

    #[cfg(feature = "rqrr")]
    all_decoders.push(Box::new(decoders::RqrrDecoder));

    #[cfg(feature = "rxing")]
    all_decoders.push(Box::new(decoders::RxingDecoder));

    #[cfg(feature = "bardecoder")]
    all_decoders.push(Box::new(decoders::BardecoderDecoder));

    #[cfg(feature = "zbar")]
    all_decoders.push(Box::new(decoders::ZBarDecoder));

    let decoders: Vec<Box<dyn decoders::QrDecoder>> = if args.libs.is_empty() {
        all_decoders
    } else {
        all_decoders
            .into_iter()
            .filter(|d| args.libs.contains(&d.name().to_string()))
            .collect()
    };

    if decoders.is_empty() {
        eprintln!("No decoders matched the provided names (or the matching feature was disabled).");
        return Ok(());
    }

    println!(
        "Benchmarking libraries: {:?}",
        decoders.iter().map(|d| d.name()).collect::<Vec<_>>()
    );

    // 3. Prepare CSV Writer
    let output_csv = &args.output;
    let path = Path::new(output_csv);
    // If output file has a parent directory that doesn't exist, create it
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let should_append = path.exists()
        && std::fs::metadata(path)
            .map(|m| m.len() > 0)
            .unwrap_or(false);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(output_csv)?;

    let mut writer = csv::WriterBuilder::new()
        .has_headers(!should_append)
        .from_writer(BufWriter::new(file));

    // 4. Run Benchmark
    println!("Running benchmark with {} iterations...", args.iterations);
    let pb = ProgressBar::new((pairs.len() * decoders.len()) as u64);

    benchmark::run_benchmark(&decoders, &pairs, args.iterations, &mut writer, &pb)?;
    pb.finish_with_message("Benchmark complete");
    writer.flush()?;

    println!("Benchmark finished. Data saved to {}.", output_csv);
    println!("To generate visualizations, run: cargo run --bin analyze");

    Ok(())
}
