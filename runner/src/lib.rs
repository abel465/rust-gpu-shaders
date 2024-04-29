use structopt::StructOpt;
use strum::{Display, EnumIter, EnumString};

mod app;
mod camera;
mod context;
mod controller;
mod egui_components;
mod fps_counter;
mod model;
mod render_pass;
mod shader;
mod shaders;
mod state;
mod texture;
mod ui;
mod window;

#[derive(EnumString, EnumIter, Display, PartialEq, Eq, Copy, Clone)]
pub enum RustGPUShader {
    Mandelbrot,
    RayMarching,
    RayMarching2D,
    SierpinskiTriangle,
    KochSnowflake,
    SDFs2D,
    SDFs3D,
    HydrogenWavefunction,
    SphericalHarmonics,
    SphericalHarmonicsShape,
    FunRepDemo,
    SdfBuilder,
}

#[derive(StructOpt, Clone, Copy)]
#[structopt(name = "example-runner-wgpu")]
pub struct Options {
    #[structopt(short, long, default_value = "Mandelbrot")]
    shader: RustGPUShader,

    // Default to true after the following is fixed
    // https://github.com/gfx-rs/wgpu/issues/5128
    #[structopt(long)]
    validate_spirv: bool,
}

pub fn main() {
    let options: Options = Options::from_args();

    app::start(options);
}
