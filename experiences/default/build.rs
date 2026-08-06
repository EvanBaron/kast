use std::path::Path;
use std::process::Command;

fn compile(shader_path: &str, output_name: &str, out_dir: &str) {
    println!("cargo:rerun-if-changed={shader_path}");

    let output_path = Path::new(out_dir).join(output_name);
    let status = Command::new("glslangValidator")
        .args(["-V", "-o"])
        .arg(&output_path)
        .arg(shader_path)
        .status()
        .expect("failed to run glslangValidator (are you in the nix shell?)");

    if !status.success() {
        panic!("shader compilation failed for {shader_path}");
    }
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    compile("shaders/triangle.vert", "triangle.vert.spv", &out_dir);
    compile("shaders/triangle.frag", "triangle.frag.spv", &out_dir);
}
