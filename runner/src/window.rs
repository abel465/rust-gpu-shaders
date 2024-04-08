use crate::{shader::CompiledShaderModules, RustGPUShader};
use egui_winit::winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoop, EventLoopBuilder},
    window::{self, WindowBuilder},
};

pub enum UserEvent {
    NewModule(RustGPUShader, CompiledShaderModules),
    SwitchShader(RustGPUShader),
    NewBuffersReady,
    SetVSync(bool),
}

pub struct Window {
    pub event_loop: EventLoop<UserEvent>,
    pub window: window::Window,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
        let window = WindowBuilder::new()
            .with_title("Rust GPU Shaders")
            .with_inner_size(PhysicalSize::new(1280.0, 720.0))
            .build(&event_loop)
            .unwrap();

        Self { event_loop, window }
    }
}
