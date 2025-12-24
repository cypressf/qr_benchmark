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

struct Verifier;

impl Verifier {
    fn distance(p1: (f32, f32), p2: (f32, f32)) -> f32 {
        ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()
    }

    fn check_points(
        expected_sets: &[Vec<(f32, f32)>],
        actual: &[(f32, f32)],
        tolerance: f32,
    ) -> bool {
        if actual.len() != 4 {
            return false;
        }

        for expected in expected_sets {
            if expected.len() != 4 {
                continue;
            }

            // Check all 4 rotations
            let mut min_avg_dist = f32::MAX;
            for offset in 0..4 {
                let mut total_dist = 0.0;
                for i in 0..4 {
                    total_dist += Self::distance(expected[i], actual[(i + offset) % 4]);
                }
                min_avg_dist = min_avg_dist.min(total_dist / 4.0);
            }

            if min_avg_dist < tolerance {
                return true;
            }
        }
        false
    }

    fn normalize_text(text: &str) -> String {
        text.replace("\r\n", "\n").trim().to_string()
    }

    fn verify(pair: &TestPair, result: &crate::decoders::DecodeResult) -> String {
        if let Some(expected_text) = &pair.expected_text {
            let normalized_expected = Self::normalize_text(expected_text);
            let normalized_decoded = Self::normalize_text(&result.text);
            if normalized_decoded == normalized_expected {
                return "Correct".to_string();
            }
        } else if let Some(expected_points) = &pair.expected_points {
            if let Some(actual_points) = &result.points {
                if Self::check_points(expected_points, actual_points, 50.0) {
                    return "Correct".to_string();
                }
            } else {
                return "NoPoints".to_string();
            }
        }
        "Incorrect".to_string()
    }
}

pub fn run_benchmark<W: std::io::Write>(
    decoders: &[Box<dyn QrDecoder>],
    pairs: &[TestPair],
    iterations: u32,
    writer: &mut csv::Writer<W>,
    progress: &ProgressBar,
) -> Result<()> {
    for pair in pairs {
        let img = match image::open(&pair.image_path) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Failed to load image {:?}: {}", pair.image_path, e);
                progress.inc(decoders.len() as u64);
                continue;
            }
        };

        for decoder in decoders {
            // Warmup
            let _ = decoder.decode(&img);

            for i in 1..=iterations {
                let start = Instant::now();
                let result = decoder.decode(&img);
                let duration = start.elapsed().as_micros();

                let (status, decoded_text) = match result {
                    Ok(ref res) => (Verifier::verify(pair, res), res.text.clone()),
                    Err(_) => ("Failed".to_string(), "".to_string()),
                };

                let record = Measurement {
                    library: decoder.name().to_string(),
                    category: pair.category.clone(),
                    file_path: pair.image_path.to_string_lossy().to_string(),
                    iteration: i,
                    duration_us: duration,
                    status,
                    expected_text: pair.expected_text.clone().unwrap_or_else(|| "POINTS".to_string()),
                    decoded_text,
                };

                writer.serialize(record)?;
            }
            progress.inc(1);
        }
    }
    Ok(())
}
