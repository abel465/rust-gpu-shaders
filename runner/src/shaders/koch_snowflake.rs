use crate::window::UserEvent;
use bytemuck::Zeroable;
use egui::Context;
use glam::{vec2, Vec2};
use shared::{from_pixels, push_constants::koch_snowflake::ShaderConstants};
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoopProxy,
};

pub struct Controller {
    size: PhysicalSize<u32>,
    cursor: Vec2,
    use_antisnowflake: bool,
    shader_constants: ShaderConstants,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            cursor: Vec2::ZERO,
            use_antisnowflake: false,
            shader_constants: ShaderConstants::zeroed(),
        }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            cursor: from_pixels(self.cursor, self.size.into()).into(),
            use_antisnowflake: self.use_antisnowflake as u32,
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut egui::Ui, _: &EventLoopProxy<UserEvent>) {
        ui.radio_value(&mut self.use_antisnowflake, false, "Snowflake");
        ui.radio_value(&mut self.use_antisnowflake, true, "AntiSnowflake");
    }
}
