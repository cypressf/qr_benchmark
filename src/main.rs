use anyhow::Result;
use indicatif::ProgressBar;
use qr_benchmark::{benchmark, data, decoders};
use std::fs::File;
use std::io::BufWriter;

const OUTPUT_CSV: &str = "raw_measurements.csv";
const ITERATIONS: u32 = 5;
const DATA_DIRS: &[&str] = &["./qrcodes/decoding", "./qrcodes/detection"];

fn main() -> Result<()> {
    // 1. Data Discovery
    println!("Discovering test data from {:?}...", DATA_DIRS);
    let pairs = data::discover_test_data(DATA_DIRS, None)?;
    println!("Found {} test pairs.", pairs.len());

    // 2. Setup Decoders
    let decoders: Vec<Box<dyn decoders::QrDecoder>> = vec![
        Box::new(decoders::RqrrDecoder),
        Box::new(decoders::RxingDecoder),
        Box::new(decoders::BardecoderDecoder),
        Box::new(decoders::ZBarDecoder),
    ];

    // 3. Prepare CSV Writer
    let file = File::create(OUTPUT_CSV)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));

    // 4. Run Benchmark
    println!("Running benchmark with {} iterations...", ITERATIONS);
    let pb = ProgressBar::new((pairs.len() * decoders.len()) as u64);

    benchmark::run_benchmark(&decoders, &pairs, ITERATIONS, &mut writer, &pb)?;
    pb.finish_with_message("Benchmark complete");
    writer.flush()?;

    println!("Benchmark finished. Data saved to {}.", OUTPUT_CSV);
    println!("To generate visualizations, run: cargo run --bin analyze");

    Ok(())
}
