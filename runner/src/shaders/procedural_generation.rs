use bytemuck::Zeroable;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
};
use glam::{vec2, Vec2};
use shared::push_constants::procedural_generation::ShaderConstants;
use std::time::Instant;

pub struct Controller {
    size: PhysicalSize<u32>,
    cursor: Vec2,
    prev_cursor: Vec2,
    camera: Vec2,
    mouse_button_pressed: bool,
    shader_constants: ShaderConstants,
    start: Instant,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            camera: Vec2::ZERO,
            mouse_button_pressed: false,
            shader_constants: ShaderConstants::zeroed(),
            start: Instant::now(),
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_button_pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };
        }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
        if self.mouse_button_pressed {
            self.camera -= self.cursor - self.prev_cursor
        }
        self.prev_cursor = self.cursor;
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        false
    }
}
