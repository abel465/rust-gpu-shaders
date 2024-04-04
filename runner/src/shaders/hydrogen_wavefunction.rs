use crate::window::UserEvent;
use bytemuck::Zeroable;
use egui::Context;
use glam::{vec2, Vec2};
use shared::push_constants::hydrogen_wavefunction::ShaderConstants;
use std::time::Instant;
use egui_winit::winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    cursor: Vec2,
    camera: Vec2,
    camera_distance: f32,
    mouse_button_pressed: bool,
    shader_constants: ShaderConstants,
    n: i32,
    l: i32,
    m: i32,
    root: i32,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            cursor: Vec2::ZERO,
            camera: Vec2::ZERO,
            camera_distance: 30.0,
            mouse_button_pressed: false,
            shader_constants: ShaderConstants::zeroed(),
            n: 1,
            l: 0,
            m: 0,
            root: 2,
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

    fn mouse_delta(&mut self, delta: (f64, f64)) {
        if self.mouse_button_pressed {
            self.camera -= vec2(delta.0 as f32, delta.1 as f32);
        }
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
        self.camera_distance *= scroll;
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            cursor: self.cursor.into(),
            camera_distance: self.camera_distance,
            translate: (self.camera / self.size.height as f32).into(),
            mouse_button_pressed: !(1 << self.mouse_button_pressed as u32),
            n: self.n as u32,
            l: self.l as u32,
            m: self.m,
            root: self.root,
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
            ui.label("root:");
            ui.add(
                egui::DragValue::new(&mut self.root)
                    .clamp_range(1..=6)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("n:");
            ui.add(
                egui::DragValue::new(&mut self.n)
                    .clamp_range(1..=5)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("l:");
            ui.add(
                egui::DragValue::new(&mut self.l)
                    .clamp_range(0..=self.n - 1)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("m:");
            ui.add(
                egui::DragValue::new(&mut self.m)
                    .clamp_range(-self.l..=self.l)
                    .speed(0.1),
            );
        });
    }
}
