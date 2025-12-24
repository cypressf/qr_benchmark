mod benchmark;
mod data;
mod decoders;
mod viz;

use anyhow::Result;
use indicatif::ProgressBar;
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<()> {
    let output_csv = "raw_measurements.csv";

    // 1. Data Discovery
    println!("Discovering test data...");
    // Limit to None for full run.
    let pairs = data::discover_test_data(&["../decoding", "../detection"], None)?;
    println!("Found {} test pairs.", pairs.len());

    // 2. Setup Decoders
    let decoders: Vec<Box<dyn decoders::QrDecoder>> = vec![
        Box::new(decoders::RqrrDecoder),
        Box::new(decoders::RxingDecoder),
        Box::new(decoders::BardecoderDecoder),
    ];

    // 3. Prepare CSV Writer
    let file = File::create(output_csv)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));

    // 4. Run Benchmark
    println!("Running benchmark...");
    let iterations = 5; // Reduced for speed

    let pb = ProgressBar::new((pairs.len() * decoders.len()) as u64);

    benchmark::run_benchmark(&decoders, &pairs, iterations, &mut writer, &pb)?;
    pb.finish_with_message("Benchmark complete");
    writer.flush()?;

    // 5. Visualization
    println!("Generating visualizations...");
    viz::generate_plots(output_csv)?;
    println!("Done! Check {} and generated PNGs.", output_csv);

    Ok(())
}
