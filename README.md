# Rust GPU Shaders
![Screenshot](docs/images/screenshot_sdfs_2d.jpg)

A collection of interactive graphics programs

Uses
[rust-gpu](https://github.com/EmbarkStudios/rust-gpu),
[wgpu](https://github.com/gfx-rs/wgpu), and
[egui](https://github.com/emilk/egui)

## Usage

```bash
nix run github:abel465/rust-gpu-shaders
```

## Development
Shader hot reloading is enabled
```bash
git clone https://github.com/abel465/rust-gpu-shaders.git
cd rust-gpu-shaders
nix develop
cargo run --release
```
