use anyhow::Result;
use qr_benchmark::decoders::{BardecoderDecoder, QrDecoder, RqrrDecoder, RxingDecoder};
use serde::Serialize;
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize)]
struct DetectionResult {
    library: String,
    points: Option<Vec<(f32, f32)>>,
    text: String,
    status: String,
}

#[derive(Serialize)]
struct DebugOutput {
    image_path: String,
    ground_truth_sets: Option<Vec<Vec<(f32, f32)>>>,
    detections: Vec<DetectionResult>,
}

fn main() -> Result<()> {
    // Hardcoded test image for now, or take from args
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: debug_detect <image_path>");
        return Ok(());
    }
    let image_path = PathBuf::from(&args[1]);

    println!("Processing: {:?}", image_path);
    let img = image::open(&image_path).expect("Failed to open image");
    println!("Image dimensions: {}x{}", img.width(), img.height());

    // Try to load ground truth
    let txt_path = image_path.with_extension("txt");
    let mut ground_truth_sets = None;
    if txt_path.exists() {
        let content = std::fs::read_to_string(&txt_path)?;
        let mut sets = Vec::new();
        let mut current_set = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") || line == "SETS" {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();

            // Format 1: All 4 points (8 coordinates) on one line
            if parts.len() >= 8 {
                let mut points = Vec::new();
                for i in 0..4 {
                    if let (Ok(x), Ok(y)) =
                        (parts[i * 2].parse::<f32>(), parts[i * 2 + 1].parse::<f32>())
                    {
                        points.push((x, y));
                    }
                }
                if points.len() == 4 {
                    sets.push(points);
                }
            }
            // Format 2: One point (2 coordinates) per line
            else if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    current_set.push((x, y));
                    if current_set.len() == 4 {
                        sets.push(current_set.clone());
                        current_set.clear();
                    }
                }
            }
        }

        if !sets.is_empty() {
            ground_truth_sets = Some(sets);
        }
    }

    let decoders: Vec<Box<dyn QrDecoder>> = vec![
        Box::new(RqrrDecoder),
        Box::new(RxingDecoder),
        Box::new(BardecoderDecoder),
    ];

    let mut detections = Vec::new();

    for decoder in decoders.iter() {
        println!("Running {}", decoder.name());
        let result = decoder.decode(&img);
        let (status, text, points) = match result {
            Ok(res) => ("Success".to_string(), res.text, res.points),
            Err(e) => ("Failed".to_string(), e.to_string(), None),
        };

        detections.push(DetectionResult {
            library: decoder.name().to_string(),
            points,
            text,
            status,
        });
    }

    let output = DebugOutput {
        image_path: image_path.to_string_lossy().to_string(),
        ground_truth_sets,
        detections,
    };

    let file = File::create("debug_output.json")?;
    serde_json::to_writer_pretty(file, &output)?;
    println!("Wrote debug_output.json");

    Ok(())
}
