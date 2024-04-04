use crate::camera::FirstPersonCamera;
use bytemuck::Zeroable;
use glam::{vec2, vec3};
use shared::push_constants::ray_marching::ShaderConstants;
use std::time::{Duration, Instant};
use egui_winit::winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta},
};

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    elapsed: Duration,
    last_elapsed: Duration,
    camera: FirstPersonCamera,
    mouse_button_pressed: bool,
    shader_constants: ShaderConstants,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            elapsed: Duration::ZERO,
            last_elapsed: Duration::ZERO,
            camera: FirstPersonCamera::new(
                size.width as f32 / size.height as f32,
                vec3(0.0, 1.0, 1.0),
                1.0,
            ),
            mouse_button_pressed: false,
            shader_constants: ShaderConstants::zeroed(),
        }
    }

    fn keyboard_input(&mut self, input: KeyboardInput) {
        self.camera.keyboard_input(input);
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_button_pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };
        }
    }

    fn mouse_delta(&mut self, delta: (f64, f64)) {
        let translate = vec2(delta.0 as f32, delta.1 as f32) / self.size.height as f32;
        self.camera.rotate(translate);
    }

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let zoom = match delta {
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
        self.camera.zoom(zoom);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.camera.resize(size);
    }

    fn update(&mut self) {
        self.elapsed = self.start.elapsed();
        self.camera.update(self.elapsed - self.last_elapsed);
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.elapsed.as_secs_f32(),
            pos: self.camera.position.into(),
            yaw: self.camera.yaw,
            pitch: self.camera.pitch,
        };
        self.last_elapsed = self.elapsed;
    }

    fn cursor_visible(&self) -> bool {
        false
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }
}
