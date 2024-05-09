use crate::model::Vertex;
use crate::window::UserEvent;
use crate::{shaders, RustGPUShader};
use egui::{Context, Ui};
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};
use shaders::*;

pub enum BindGroupBufferType<'a> {
    #[allow(dead_code)]
    Uniform(Uniform<'a>),
    SSBO(SSBO<'a>),
}

pub struct SSBO<'a> {
    pub data: &'a [u8],
    pub read_only: bool,
}

pub struct Uniform<'a> {
    pub data: &'a [u8],
}

#[derive(Default)]
pub struct BufferData<'a> {
    pub vertex: Option<&'a [Vertex]>,
    pub index: Option<&'a [u32]>,
    pub bind_group_buffers: Vec<BindGroupBufferType<'a>>,
    pub use_depth_buffer: bool,
}

pub trait Controller {
    fn new(size: PhysicalSize<u32>) -> Self
    where
        Self: Sized;
    fn keyboard_input(&mut self, _input: KeyEvent) {}
    fn mouse_input(&mut self, _state: ElementState, _button: MouseButton) {}
    fn mouse_move(&mut self, _position: PhysicalPosition<f64>) {}
    fn mouse_delta(&mut self, _delta: (f64, f64)) {}
    fn mouse_scroll(&mut self, _delta: MouseScrollDelta) {}
    fn resize(&mut self, size: PhysicalSize<u32>);
    fn update(&mut self);
    fn push_constants(&self) -> &[u8];
    fn ui(&mut self, _ctx: &Context, _ui: &mut Ui, _event_proxy: &EventLoopProxy<UserEvent>) {}
    fn has_ui(&self) -> bool {
        false
    }
    fn buffers(&self) -> BufferData {
        BufferData::default()
    }
    fn cursor_visible(&self) -> bool {
        true
    }
}

pub fn new_controller(shader: RustGPUShader, size: PhysicalSize<u32>) -> Box<dyn Controller> {
    match shader {
        RustGPUShader::Mandelbrot => Box::new(mandelbrot::Controller::new(size)),
        RustGPUShader::RayMarching => Box::new(ray_marching::Controller::new(size)),
        RustGPUShader::RayMarching2D => Box::new(ray_marching_2d::Controller::new(size)),
        RustGPUShader::SierpinskiTriangle => Box::new(sierpinski_triangle::Controller::new(size)),
        RustGPUShader::KochSnowflake => Box::new(koch_snowflake::Controller::new(size)),
        RustGPUShader::SDFs2D => Box::new(sdfs_2d::Controller::new(size)),
        RustGPUShader::SDFs3D => Box::new(sdfs_3d::Controller::new(size)),
        RustGPUShader::HydrogenWavefunction => {
            Box::new(hydrogen_wavefunction::Controller::new(size))
        }
        RustGPUShader::SphericalHarmonics => Box::new(spherical_harmonics::Controller::new(size)),
        RustGPUShader::SphericalHarmonicsShape => {
            Box::new(spherical_harmonics_shape::Controller::new(size))
        }
        RustGPUShader::FunRepDemo => Box::new(fun_rep_demo::Controller::new(size)),
        RustGPUShader::SdfBuilder => Box::new(sdf_builder::Controller::new(size)),
    }
}
