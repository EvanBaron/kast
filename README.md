# Kast

A game engine written in Rust with raw Vulkan bindings and Winit, that's it.

## Prerequisites

- **Rust**: Latest stable version.
- **Vulkan SDK**: Ensure Vulkan headers and loader are installed.
- **Nix** (Optional): A `shell.nix` is provided for a consistent development environment.

## Development Environment

This project uses `shell.nix` and is compatible with `direnv`. If you have `direnv` installed, simply run:

```bash
direnv allow
```

Otherwise, you can enter the shell manually:

```bash
nix-shell
```

## Setup & Dependencies

The project relies on Vulkan Headers and `glslangValidator` (for shader compilation).

### 1. Vulkan Headers
This project generates raw bindings from the Vulkan C headers.
```bash
git clone https://github.com/KhronosGroup/Vulkan-Headers
cd Vulkan-Headers && cmake -DCMAKE_INSTALL_PREFIX=../build_tools . && cmake --build . && cmake --install .
```

### 2. GLSLang (Shader Compiler)
Required to compile GLSL shaders into SPIR-V.
```bash
git clone https://github.com/KhronosGroup/glslang
cd glslang && \
./update_glslang_sources.py && \
cmake -B ./build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../build_tools && \
cd build && \
cmake --build . --config Release --target install
```

## Running

To run the application:

```bash
cargo run -p kast
```

## Project Structure

- `core/`: The main game engine logic, renderer, and scene management.
- `vk_bindings/`: Custom `bindgen` generation of Vulkan C headers.
- `shaders/`: Compiled SPIR-V shaders (runtime).
- `shader_src/`: Source GLSL shaders.

## Resources used

[Vulkan Rust Tutorial by Lóránt Seres](https://vulkan-rust-tutorial.gitlab.io/)
