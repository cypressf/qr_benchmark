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

// Calculate intersection over union for polygons if possible, 
// or just average corner distance error.
// Since ordering of points might differ or be rotated, we need to be careful.
// A simpler metric for "Correct" detection:
// Average distance between matched corners is less than a threshold (e.g. 5% of image size or fixed pixels).
fn is_detection_correct(
    expected_points_sets: &[Vec<(f32, f32)>], 
    actual_points: &[(f32, f32)],
    tolerance: f32
) -> bool {
    if actual_points.len() != 4 {
        return false;
    }

    // Check against ALL expected sets. If it matches ANY set, it's a success.
    for expected_points in expected_points_sets {
         if expected_points.len() != 4 {
             continue; 
         }

        // We don't know the starting corner index for sure (rotation).
        // Try all 4 rotations for the actual points to find best match.
        let mut min_avg_dist = f32::MAX;

        for offset in 0..4 {
            let mut total_dist = 0.0;
            for i in 0..4 {
                let p1 = expected_points[i];
                let p2 = actual_points[(i + offset) % 4];
                let dist = ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt();
                total_dist += dist;
            }
            let avg_dist = total_dist / 4.0;
            if avg_dist < min_avg_dist {
                min_avg_dist = avg_dist;
            }
        }
        
        if min_avg_dist < tolerance {
            return true;
        }
    }

    false
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
                    Ok(decode_result) => {
                        let mut status = "Incorrect".to_string();
                        let text = decode_result.text.clone();

                        // 1. Text Comparison (if expected text is available)
                        if let Some(expected_text) = &pair.expected_text {
                            let normalized_expected = expected_text.replace("\r\n", "\n").trim().to_string();
                            let normalized_decoded = text.replace("\r\n", "\n").trim().to_string();
                            
                            if normalized_decoded == normalized_expected {
                                status = "Correct".to_string();
                            }
                        } 
                        // 2. Point/Detection Comparison (if expected points are available)
                        else if let Some(expected_points_sets) = &pair.expected_points {
                            if let Some(actual_points) = &decode_result.points {
                                // Use a tolerance of 50.0 pixels (generous but ensures general alignment)
                                if is_detection_correct(expected_points_sets, actual_points, 50.0) {
                                    status = "Correct".to_string();
                                }
                            } else {
                                status = "NoPoints".to_string();
                            }
                        }

                        (status, text)
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
