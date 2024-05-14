use crate::window::UserEvent;
use bytemuck::Zeroable;
use egui::Context;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};
use glam::{vec2, Vec2};
use shared::push_constants::mandelbrot::ShaderConstants;

pub struct Controller {
    size: PhysicalSize<u32>,
    cursor: Vec2,
    prev_cursor: Vec2,
    camera: Vec2,
    zoom: f32,
    mouse_button_pressed: bool,
    exponent: f32,
    num_iterations: u32,
    shader_constants: ShaderConstants,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            camera: Vec2::ZERO,
            zoom: 1.0,
            mouse_button_pressed: false,
            exponent: 2.0,
            num_iterations: 35,
            shader_constants: ShaderConstants::zeroed(),
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

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let scroll = match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let v = 1.0 + 0.1 * y.abs();
                if y < 0.0 {
                    v
                } else {
                    1.0 / v
                }
            }
            MouseScrollDelta::PixelDelta(p) => {
                let v = 1.0 + 0.02 * (1.0 + p.y.abs() as f32).ln();
                if p.y < 0.0 {
                    v
                } else {
                    1.0 / v
                }
            }
        };
        self.zoom *= scroll;
        self.camera *= 1.0 / scroll;
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            zoom: self.zoom,
            translate: self.camera.into(),
            exponent: self.exponent,
            num_iterations: self.num_iterations,
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut egui::Ui, _: &EventLoopProxy<UserEvent>) {
        ui.horizontal(|ui| {
            ui.label("Exponent:");
            ui.add(
                egui::DragValue::new(&mut self.exponent)
                    .clamp_range(1.0..=6.0)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Num Iterations:");
            ui.add(
                egui::DragValue::new(&mut self.num_iterations)
                    .clamp_range(2..=200)
                    .speed(1),
            );
        });
    }
}
