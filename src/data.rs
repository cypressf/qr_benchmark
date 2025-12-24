use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TestPair {
    pub image_path: PathBuf,
    pub category: String,
    pub expected_text: String,
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
                        // Determine category from parent dir
                        let category = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Check limit
                        if let Some(limit) = limit_per_category {
                            let count = category_counts.entry(category.clone()).or_insert(0);
                            if *count >= limit {
                                continue;
                            }
                            *count += 1;
                        }

                        let text_path = path.with_extension("txt");
                        if text_path.exists() {
                            let expected_text = std::fs::read_to_string(&text_path)?;
                            // Clean up expected text (trim whitespace)
                            // IMPORTANT: Many text editors add a newline at end of file,
                            // but QR decoders might output raw string without it.
                            // Or the QR content actually has newlines.
                            // We should probably normalize line endings too (\r\n vs \n).
                            let expected_text = expected_text.trim().replace("\r\n", "\n");

                            if expected_text.starts_with("#") {
                                continue;
                            }

                            pairs.push(TestPair {
                                image_path: path.to_path_buf(),
                                category,
                                expected_text,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(pairs)
}
