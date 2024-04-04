use core::f32::consts::PI;
use egui_winit::winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode},
};
use glam::{vec3, Mat4, Quat, Vec2, Vec2Swizzles, Vec3};
use std::time::Duration;

#[derive(Clone)]
pub struct RotationCamera {
    pub aspect: f32,
    pub zoom: f32,
    pub rot: Quat,
}

impl RotationCamera {
    pub fn new(aspect: f32, zoom: f32) -> Self {
        Self {
            aspect,
            zoom,
            rot: Quat::IDENTITY,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(Vec3::Z * self.zoom, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(PI / 4.0, self.aspect, 0.01, 100.0);
        let rot = Mat4::from_quat(self.rot);
        proj * view * rot
    }

    pub fn rotate(&mut self, v: Vec2) {
        let p = (v.yx() * 4.0).extend(0.0);
        self.rot = Quat::from_scaled_axis(p).mul_quat(self.rot).normalize();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
    }

    pub fn zoom(&mut self, zoom: f32) {
        self.zoom *= zoom;
    }
}

#[derive(Clone)]
pub struct FirstPersonCamera {
    pub aspect: f32,
    pub position: Vec3,
    pub zoom: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_left: f32,
    pub amount_right: f32,
}

impl FirstPersonCamera {
    pub fn new(aspect: f32, position: Vec3, zoom: f32) -> Self {
        Self {
            aspect,
            position,
            zoom,
            yaw: -PI / 2.0,
            pitch: PI,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_left: 0.0,
            amount_right: 0.0,
        }
    }

    pub fn rotate(&mut self, v: Vec2) {
        self.pitch = (self.pitch - v.y * 2.0).clamp(-PI / 3.0, PI / 3.0);
        self.yaw = self.yaw - v.x * 3.0;
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
    }

    pub fn zoom(&mut self, zoom: f32) {
        self.zoom *= zoom;
    }

    pub fn update(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = vec3(-yaw_sin, 0.0, -yaw_cos).normalize();
        let right = vec3(yaw_cos, 0.0, -yaw_sin).normalize();

        let translate = forward * (self.amount_forward - self.amount_backward)
            + right * (self.amount_right - self.amount_left);
        self.position += translate.normalize_or_zero() * 4.0 * dt;
    }

    pub fn keyboard_input(&mut self, input: KeyboardInput) {
        let pressed = input.state == ElementState::Pressed;
        let amount = if pressed { 1.0 } else { 0.0 };
        if let Some(keycode) = input.virtual_keycode {
            match keycode {
                VirtualKeyCode::W => {
                    self.amount_forward = amount;
                    if pressed {
                        self.amount_backward = 0.0;
                    }
                }
                VirtualKeyCode::S => {
                    self.amount_backward = amount;
                    if pressed {
                        self.amount_forward = 0.0;
                    }
                }
                VirtualKeyCode::A => {
                    self.amount_left = amount;
                    if pressed {
                        self.amount_right = 0.0;
                    }
                }
                VirtualKeyCode::D => {
                    self.amount_right = amount;
                    if pressed {
                        self.amount_left = 0.0;
                    }
                }
                _ => {}
            }
        }
    }
}
