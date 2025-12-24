use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let root_dirs = vec!["../decoding", "../detection"];
    let mut count = 0;
    
    // We are looking for index 173 (0-indexed), which is the 174th image.
    let target_index = 173;

    for root in root_dirs {
        for entry in WalkDir::new(root).follow_links(true) {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext = ext.to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        // Logic from data.rs: Check if txt file exists
                        let text_path = path.with_extension("txt");
                        if text_path.exists() {
                            if count == target_index {
                                println!("Found failing image at index {}: {}", count, path.display());
                                return;
                            }
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    println!("Could not find image at index {}", target_index);
}

