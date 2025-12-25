use anyhow::Result;
use image::DynamicImage;
use indicatif::ProgressBar;
use qr_benchmark::{
    benchmark, data,
    decoders::{DecodeResult, QrDecoder},
};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;

// Example dummy decoder
struct MyDecoder;

impl QrDecoder for MyDecoder {
    fn name(&self) -> &'static str {
        "my_decoder"
    }

    fn decode(&self, _image: &DynamicImage) -> Result<DecodeResult> {
        // Implement your decoding logic here.
        // For this example, we just return an error so we don't need real decoding logic.
        Err(anyhow::anyhow!("Not implemented"))
    }
}

fn main() -> Result<()> {
    // 1. Data Discovery
    // Note: You need to point this to where the test data is.
    // If running from the root of the repo, this path should work.
    let data_dirs = &["./qrcodes/decoding", "./qrcodes/detection"];

    // Check if directories exist
    if !Path::new(data_dirs[0]).exists() {
        eprintln!("Error: Test data not found at ./qrcodes. Please download the dataset.");
        return Ok(());
    }

    println!("Discovering test data from {:?}...", data_dirs);
    let pairs = data::discover_test_data(data_dirs, None)?;
    println!("Found {} test pairs.", pairs.len());

    // 2. Setup Decoders
    let decoders: Vec<Box<dyn QrDecoder>> = vec![Box::new(MyDecoder)];

    // 3. Prepare CSV Writer
    let output_csv = "my_benchmark.csv";
    let path = Path::new(output_csv);
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
    let iterations = 1;
    println!("Running benchmark with {} iterations...", iterations);
    let pb = ProgressBar::new((pairs.len() * decoders.len()) as u64);

    benchmark::run_benchmark(&decoders, &pairs, iterations, &mut writer, &pb)?;
    pb.finish_with_message("Benchmark complete");
    writer.flush()?;

    println!("Benchmark finished. Data saved to {}.", output_csv);

    Ok(())
}
