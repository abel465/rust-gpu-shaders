use bytemuck::Zeroable;
use glam::vec2;
use shared::push_constants::sierpinski_triangle::ShaderConstants;
use egui_winit::winit::{dpi::PhysicalSize, event::MouseScrollDelta};

pub struct Controller {
    size: PhysicalSize<u32>,
    scroll: f64,
    zoom: f32,
    shader_constants: ShaderConstants,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            scroll: 0.0,
            zoom: 0.0,
            shader_constants: ShaderConstants::zeroed(),
        }
    }

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.scroll += match delta {
            MouseScrollDelta::LineDelta(_, y) => y as f64,
            MouseScrollDelta::PixelDelta(p) => 0.15 * (1.0 + p.y.abs()).ln() * p.y.signum(),
        };
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        let c = 59.87868500430847;
        let v = 34.102688577484;
        let scroll = if self.scroll > c {
            self.scroll - v * (1.0 + ((self.scroll - c) / v).floor())
        } else if self.scroll < -1.0 {
            -1.0 - (-self.scroll).log10()
        } else {
            self.scroll
        };
        self.zoom = 0.85_f64.powf(scroll) as f32;

        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            zoom: self.zoom,
            dim: vec2(-0.08443636, -0.087451585).into(),
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }
}
