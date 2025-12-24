use anyhow::Result;
use qr_benchmark::viz;
use std::env;

fn main() -> Result<()> {
    // Check if a CSV file was passed as argument, otherwise use default
    let args: Vec<String> = env::args().collect();
    let output_csv = if args.len() > 1 {
        &args[1]
    } else {
        "raw_measurements.csv"
    };

    println!("Generating visualizations from {}...", output_csv);
    viz::generate_plots(output_csv)?;
    println!("Done! Check generated PNGs.");

    Ok(())
}

