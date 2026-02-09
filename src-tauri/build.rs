use std::fs;
use std::path::Path;

fn main() {
    tauri_build::build();

    // Copy public folder to target directory for development
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let public_dir = Path::new("../public");
    let target_public_dir = Path::new(&out_dir).join("public");

    if public_dir.exists() {
        // Create target directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&target_public_dir) {
            eprintln!("Warning: Failed to create target public directory: {}", e);
            return;
        }

        // Copy all files from public to target
        if let Ok(entries) = fs::read_dir(public_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let src = entry.path();
                    let dst = target_public_dir.join(entry.file_name());

                    // Remove destination file if it exists to avoid permission errors
                    if dst.exists() {
                        let _ = fs::remove_file(&dst);
                    }

                    if let Err(e) = fs::copy(&src, &dst) {
                        eprintln!("Warning: Failed to copy {}: {}", src.display(), e);
                    } else {
                        println!("cargo:rerun-if-changed={}", src.display());
                    }
                }
            }
        }
    }
}
