use bytemuck::Zeroable;
use egui::{vec2, Vec2};
use shared::push_constants::ray_marching_2d::ShaderConstants;
use std::time::{Duration, Instant};
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
};

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    elapsed: Duration,
    cursor: Vec2,
    drag_start: Vec2,
    drag_end: Vec2,
    camera: Vec2,
    zoom: f32,
    mouse_button_pressed: bool,

    scroll: f32,
    drag: Vec2,
    prev_cursor: Vec2,
    shader_constants: ShaderConstants,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            elapsed: Duration::ZERO,
            cursor: Vec2::ZERO,
            drag_start: Vec2::ZERO,
            drag_end: Vec2::ZERO,
            camera: Vec2::ZERO,
            zoom: 1.0,
            mouse_button_pressed: false,

            scroll: 1.0,
            drag: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            shader_constants: ShaderConstants::zeroed(),
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_button_pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => {
                    self.drag = self.drag_start - self.drag_end;
                    false
                }
            };

            self.drag_start = self.cursor;
            self.drag_end = self.cursor;
        }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
        if self.mouse_button_pressed {
            self.drag_end = self.cursor;
        }
    }

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.scroll *= match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let v = 1.0 + 0.1 * y.abs();
                if y < 0.0 {
                    v
                } else {
                    1.0 / v
                }
            }
            MouseScrollDelta::PixelDelta(p) => {
                let v = 1.0 + 0.1 * (1.0 + p.y.abs() as f32).ln();
                if p.y < 0.0 {
                    v
                } else {
                    1.0 / v
                }
            }
        };
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.elapsed = self.start.elapsed();
        self.zoom *= self.scroll;
        self.camera *= 1.0 / self.scroll;
        self.camera += self.drag;
        self.shader_constants = ShaderConstants {
            width: self.size.width,
            height: self.size.height,
            time: self.elapsed.as_secs_f32(),
            cursor_x: self.cursor.x,
            cursor_y: self.cursor.y,
            drag_start_x: self.drag_start.x,
            drag_start_y: self.drag_start.y,
            drag_end_x: self.drag_end.x,
            drag_end_y: self.drag_end.y,
            zoom: self.zoom,
            translate_x: self.camera.x,
            translate_y: self.camera.y,
            mouse_button_pressed: !(1 << self.mouse_button_pressed as u32),
        };
        self.finish_update();
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }
}

impl Controller {
    pub fn finish_update(&mut self) {
        self.scroll = 1.0;
        self.drag = Vec2::ZERO;
        self.prev_cursor = self.cursor;
    }
}
