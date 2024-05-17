use crate::window::UserEvent;
use bytemuck::Zeroable;
use egui::Context;
use egui_winit::winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};
use glam::{vec2, Vec2};
use shared::{push_constants::hydrogen_wavefunction::ShaderConstants, spherical_harmonics};
use std::time::Instant;

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    paused: Instant,
    cursor: Vec2,
    camera: Vec2,
    camera_distance: f32,
    mouse_button_pressed: bool,
    shader_constants: ShaderConstants,
    n: i32,
    l: i32,
    m: i32,
    brightness: f32,
    time_dependent: bool,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            paused: Instant::now(),
            cursor: Vec2::ZERO,
            camera: Vec2::ZERO,
            camera_distance: 30.0,
            mouse_button_pressed: false,
            shader_constants: ShaderConstants::zeroed(),
            n: 4,
            l: 1,
            m: 1,
            brightness: 2.0,
            time_dependent: false,
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
        let n = self.n as u32;
        let l = self.l.clamp(0, n as i32 - 1) as u32;
        let m = self.m.clamp(-(l as i32), l as i32);
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: (if self.time_dependent {
                self.start.elapsed()
            } else {
                self.paused - self.start
            })
            .as_secs_f32(),
            cursor: self.cursor.into(),
            camera_distance: self.camera_distance,
            translate: (self.camera / self.size.height as f32).into(),
            mouse_button_pressed: !(1 << self.mouse_button_pressed as u32),
            n,
            l,
            m,
            brightness: self.brightness,
            normalization_constant: radial_nc(n, l) * angular_nc(m, l),
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut egui::Ui, _: &EventLoopProxy<UserEvent>) {
        if ui
            .checkbox(&mut self.time_dependent, "Evolve over time")
            .clicked()
        {
            if self.time_dependent {
                self.start += self.paused.elapsed();
            } else {
                self.paused = Instant::now();
            }
        }
        ui.add(egui::Slider::new(&mut self.brightness, 0.0..=5.0).text("Brightness"));
        ui.add(egui::Slider::new(&mut self.n, 1..=5).text("n"));
        ui.add(egui::Slider::new(&mut self.l, 0..=self.n - 1).text("l"));
        ui.add(egui::Slider::new(&mut self.m, -self.l..=self.l).text("m"));
    }
}

fn radial_nc(n: u32, l: u32) -> f32 {
    use spherical_harmonics::factorialu;
    ((2.0 / n as f32).powi(3) * factorialu(n - l - 1)
        / (2.0 * n as f32 * factorialu(n + l).powi(3)))
    .sqrt()
}

fn angular_nc(m: i32, l: u32) -> f32 {
    spherical_harmonics::normalization_constant(m, l)
}
