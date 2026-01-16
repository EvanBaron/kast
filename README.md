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

## Running

To run the application:

```bash
cargo run -p core
```

## Project Structure

- `core/`: The main application logic and Vulkan renderer.
- `vk_bindings/`: Low-level Vulkan bindings generated from headers.
