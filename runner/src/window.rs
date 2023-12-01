use crate::shader::CompiledShaderModules;

use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoop, EventLoopBuilder},
    window::{self, WindowBuilder},
};

pub struct Window {
    pub event_loop: EventLoop<CompiledShaderModules>,
    pub window: window::Window,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title("Rust GPU Shaders")
            .with_inner_size(PhysicalSize::new(1280.0, 720.0))
            .build(&event_loop)
            .unwrap();

        Self { event_loop, window }
    }
}