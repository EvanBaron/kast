use bindgen;

fn main() {
    // Generate bindings
    let include_path = "../build_tools/include/";
    let header_path = "../build_tools/include/vulkan/vulkan.h";

    // Invalidate when VK header changes
    println!("cargo:rerun-if-changed={}", header_path);

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header(header_path)
        .clang_arg(format!("-I{}", include_path))
        .use_core()
        .derive_default(true)
        .prepend_enum_name(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Save bindings
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::PathBuf::from(out_dir);
    let vk_binding_path = out_path.join("vk_bindings.rs");
    bindings
        .write_to_file(vk_binding_path)
        .expect("Couldn't write bindings!");

    // Link
    let target = std::env::var("TARGET").unwrap();
    if target == "x86_64-pc-windows-gnu" || target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-lib=vulkan-1");
    } else {
        println!("cargo:rustc-link-lib=vulkan");
    }
}
