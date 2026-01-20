use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Visit all directories recursively and push all files to a vector.
fn visit_dirs(dir: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                visit_dirs(&path, paths)?;
            } else {
                paths.push(path);
            }
        }
    }

    Ok(())
}

fn main() {
    let glslang_path = Path::new("../build_tools/bin/glslangValidator");
    let shader_src_dir_path = Path::new("../shader_src");
    let shader_out_dir_path = Path::new("../shaders");

    // Ensure output directory exists
    if !shader_out_dir_path.exists() {
        fs::create_dir_all(&shader_out_dir_path)
            .expect("Failed to create shaders output directory");
    }

    println!("cargo:rerun-if-changed={}", shader_src_dir_path.display());

    let mut shader_paths = Vec::new();
    visit_dirs(&shader_src_dir_path, &mut shader_paths).expect("Failed to read shader sources");

    for src_path in shader_paths {
        // Check for common shader extensions
        let extension = src_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !["vert", "frag"].contains(&extension) {
            continue;
        }

        let file_name = src_path.file_name().unwrap().to_str().unwrap();
        let out_name = format!("{}.spv", file_name);
        let out_path = shader_out_dir_path.join(out_name);

        println!("cargo:rerun-if-changed={}", src_path.display());

        let status = Command::new(&glslang_path)
            .arg("-V")
            .arg("-o")
            .arg(&out_path)
            .arg(&src_path)
            .status()
            .expect("Failed to run glslangValidator");

        if !status.success() {
            panic!("Shader compilation failed for: {}", src_path.display());
        }
    }
}
