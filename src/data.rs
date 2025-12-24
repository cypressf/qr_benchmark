use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

fn parse_points(content: &str) -> Option<Vec<Vec<(f32, f32)>>> {
    let mut sets = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("#") || line == "SETS" || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
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

    if sets.is_empty() {
        None
    } else {
        Some(sets)
    }
}

pub fn discover_test_data(
    root_dirs: &[&str],
    limit_per_category: Option<usize>,
) -> Result<Vec<TestPair>> {
    let mut pairs = Vec::new();
    let mut category_counts: HashMap<String, usize> = HashMap::new();

    for root in root_dirs {
        let is_detection = root.contains("detection");
        for entry in WalkDir::new(root).follow_links(true) {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e.to_lowercase(),
                None => continue,
            };

            if matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
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

                    if is_detection {
                        expected_points = parse_points(&content);
                    } else {
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

    Ok(pairs)
}
