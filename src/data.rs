use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TestPair {
    pub image_path: PathBuf,
    pub category: String,
    pub expected_text: Option<String>,
    // Optional expected points [Set1[TopLeft...], Set2[...]]
    // The ground truth can contain multiple "SETS" of points if there are multiple QRs
    // or if the format specifies it that way.
    pub expected_points: Option<Vec<Vec<(f32, f32)>>>,
}

pub fn discover_test_data(
    root_dirs: &[&str],
    limit_per_category: Option<usize>,
) -> Result<Vec<TestPair>> {
    let mut pairs = Vec::new();
    let mut category_counts: HashMap<String, usize> = HashMap::new();

    for root in root_dirs {
        for entry in WalkDir::new(root).follow_links(true) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext = ext.to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        let category = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        if let Some(limit) = limit_per_category {
                            let count = category_counts.entry(category.clone()).or_insert(0);
                            if *count >= limit {
                                continue;
                            }
                            *count += 1;
                        }

                        let text_path = path.with_extension("txt");
                        if text_path.exists() {
                            let content = std::fs::read_to_string(&text_path)?;

                            let mut expected_text = None;
                            let mut expected_points = None;

                            // Check if it's a points file (starts with # or contains coordinates)
                            // Example format:
                            // # list of hand selected 2D points
                            // SETS
                            // x1 y1 x2 y2 x3 y3 x4 y4
                            // ...

                            if content.trim().starts_with("#") || category != "decoding" {
                                // Parse points
                                let mut sets = Vec::new();

                                for line in content.lines() {
                                    let line = line.trim();
                                    if line.starts_with("#") || line == "SETS" || line.is_empty() {
                                        continue;
                                    }

                                    // Each line is a set of points for one QR code
                                    // "x1 y1 x2 y2 x3 y3 x4 y4"
                                    let parts: Vec<&str> = line.split_whitespace().collect();

                                    // We expect at least 4 pairs (8 numbers) for a quad
                                    if parts.len() >= 8 {
                                        let mut points = Vec::new();
                                        for i in 0..4 {
                                            if let (Ok(x), Ok(y)) = (
                                                parts[i * 2].parse::<f32>(),
                                                parts[i * 2 + 1].parse::<f32>(),
                                            ) {
                                                points.push((x, y));
                                            }
                                        }
                                        if points.len() == 4 {
                                            sets.push(points);
                                        }
                                    }
                                }

                                if !sets.is_empty() {
                                    expected_points = Some(sets);
                                }
                            } else {
                                // It's text content
                                expected_text = Some(content.trim().replace("\r\n", "\n"));
                            }

                            if expected_text.is_some() || expected_points.is_some() {
                                pairs.push(TestPair {
                                    image_path: path.to_path_buf(),
                                    category,
                                    expected_text,
                                    expected_points,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(pairs)
}
