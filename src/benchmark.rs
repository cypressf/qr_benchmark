use crate::data::TestPair;
use crate::decoders::QrDecoder;
use anyhow::Result;
use indicatif::ProgressBar;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct Measurement {
    pub library: String,
    pub category: String,
    pub file_path: String,
    pub iteration: u32,
    pub duration_us: u128,
    pub status: String,
    pub expected_text: String,
    pub decoded_text: String,
}

pub fn run_benchmark<W: std::io::Write>(
    decoders: &[Box<dyn QrDecoder>],
    pairs: &[TestPair],
    iterations: u32,
    writer: &mut csv::Writer<W>,
    progress: &ProgressBar,
) -> Result<()> {
    for pair in pairs {
        // Load image once
        let img = match image::open(&pair.image_path) {
            Ok(i) => i,
            Err(_) => {
                progress.inc(decoders.len() as u64);
                continue;
            }
        };

        for decoder in decoders {
            // Warmup
            let _ = decoder.decode(&img);

            // Measurements
            for i in 1..=iterations {
                let start = Instant::now();
                let result = decoder.decode(&img);
                let duration = start.elapsed().as_micros();

                let (status, decoded_text) = match result {
                    Ok(text) => {
                        if text == pair.expected_text {
                            ("Correct".to_string(), text)
                        } else {
                            ("Incorrect".to_string(), text)
                        }
                    }
                    Err(_) => ("Failed".to_string(), "".to_string()),
                };

                let record = Measurement {
                    library: decoder.name().to_string(),
                    category: pair.category.clone(),
                    file_path: pair.image_path.to_string_lossy().to_string(),
                    iteration: i,
                    duration_us: duration,
                    status,
                    expected_text: pair.expected_text.clone(),
                    decoded_text,
                };

                writer.serialize(record)?;
            }
            progress.inc(1);
        }
    }
    Ok(())
}
