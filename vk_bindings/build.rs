use bindgen;

fn main() {
    // Locate Vulkan via pkg-config
    let vulkan_lib = pkg_config::Config::new()
        .probe("vulkan")
        .expect("Failed to find Vulkan via pkg-config. Ensure vulkan-headers/loader are in your environment.");

    // Configure Bindgen
    let mut builder = bindgen::Builder::default()
        // Use a wrapper to include the standard header
        .header_contents("wrapper.h", "#include <vulkan/vulkan.h>")
        .use_core()
        .derive_default(true)
        .prepend_enum_name(false)
        .blocklist_item("None")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // Add include paths found by pkg-config
    for path in vulkan_lib.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.display()));
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        builder = builder.clang_arg("-DVK_USE_PLATFORM_WIN32_KHR");
    } else if target_os == "linux"
        || target_os == "freebsd"
        || target_os == "dragonfly"
        || target_os == "netbsd"
        || target_os == "openbsd"
    {
        if let Ok(lib) = pkg_config::Config::new().probe("x11") {
            builder = builder.clang_arg("-DVK_USE_PLATFORM_XLIB_KHR");

            for path in lib.include_paths {
                builder = builder.clang_arg(format!("-I{}", path.display()));
            }
        }

        if let Ok(lib) = pkg_config::Config::new().probe("xcb") {
            builder = builder.clang_arg("-DVK_USE_PLATFORM_XCB_KHR");

            for path in lib.include_paths {
                builder = builder.clang_arg(format!("-I{}", path.display()));
            }
        }

        if let Ok(lib) = pkg_config::Config::new().probe("wayland-client") {
            builder = builder.clang_arg("-DVK_USE_PLATFORM_WAYLAND_KHR");

            for path in lib.include_paths {
                builder = builder.clang_arg(format!("-I{}", path.display()));
            }
        }
    } else if target_os == "android" {
        builder = builder.clang_arg("-DVK_USE_PLATFORM_ANDROID_KHR");
    } else if target_os == "macos" || target_os == "ios" {
        builder = builder.clang_arg("-DVK_USE_PLATFORM_METAL_EXT");
    }

    // Handle extra clang args
    if let Ok(extra_args) = std::env::var("BINDGEN_EXTRA_CLANG_ARGS") {
        for arg in extra_args.split_whitespace() {
            builder = builder.clang_arg(arg);
        }
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

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
    }
}
