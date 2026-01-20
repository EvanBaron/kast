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

## Installation

### Vulkan Headers

```bash
git clone https://github.com/KhronosGroup/Vulkan-Headers
```

```bash
cd Vulkan-Headers && cmake -DCMAKE_INSTALL_PREFIX=build_tools && cmake --build . && cmake --install .
```

### GLSLang

```bash
git clone https://github.com/KhronosGroup/glslang
```

```bash
cd glslang && \
./update_glslang_sources.py && \
mkdir build && \
cmake -B ./build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=build_tools && \
cd build && \
cmake --build . --config Release --target install
```


## Running

To run the application:

```bash
cargo run -p core
```

## Project Structure

- `core/`: The main application logic and Vulkan renderer.
- `vk_bindings/`: Low-level Vulkan bindings generated from headers.
