use crate::window::UserEvent;
use bytemuck::Zeroable;
use egui::{Color32, Context, Rect, RichText, Sense, Stroke, Ui};
use glam::{vec2, Quat, Vec2, Vec2Swizzles};
use shared::push_constants::spherical_harmonics::{ShaderConstants, Variant};
use std::time::Instant;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    cursor: Vec2,
    prev_cursor: Vec2,
    rot: Quat,
    zoom: f32,
    mouse_button_pressed: bool,
    shader_constants: ShaderConstants,
    l: u32,
    m: i32,
    negative_m: bool,
    include_time_factor: bool,
    variant: Variant,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            rot: Quat::IDENTITY,
            zoom: 1.0,
            mouse_button_pressed: false,
            shader_constants: ShaderConstants::zeroed(),
            l: 2,
            m: 1,
            negative_m: false,
            include_time_factor: false,
            variant: Variant::Real,
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
            let translate = 4.0 * (self.prev_cursor - self.cursor) / self.size.height as f32;
            let p = self.rot * translate.yx().extend(0.0);
            self.rot = (Quat::from_scaled_axis(p) * self.rot).normalize();
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
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: if self.include_time_factor {
                self.start.elapsed().as_secs_f32()
            } else {
                0.0
            },
            zoom: self.zoom,
            l: self.l,
            m: self.m,
            rot: self.rot.into(),
            variant: self.variant as u32,
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, ctx: &Context, ui: &mut Ui, _: &EventLoopProxy<UserEvent>) {
        ui.radio_value(&mut self.variant, Variant::Real, "Real");
        ui.radio_value(&mut self.variant, Variant::Complex, "Complex");
        if ui
            .checkbox(&mut self.include_time_factor, "Include time factor")
            .clicked()
            && self.include_time_factor
        {
            self.start = Instant::now();
        }

        let (rect, response) = ui.allocate_at_least([220.0; 2].into(), Sense::drag());
        let l_max = 9;

        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let v = ((mouse_pos - rect.left_top()) * (l_max + 1) as f32 / rect.width())
                .clamp(egui::Vec2::ZERO, egui::Vec2::splat(l_max as f32));
            if v.x > v.y {
                let dif = v.x - v.y;
                self.l = (v.y + (dif / 2.0)) as u32;
                self.m = (v.x - (dif / 2.0)) as i32;
            } else {
                self.l = v.y as u32;
                self.m = v.x as i32;
            }
            ctx.input(|input| {
                if input.pointer.any_pressed() {
                    self.negative_m = input.pointer.secondary_pressed();
                }
            });
            if self.negative_m {
                self.m = -self.m;
            }
        }

        let circle_radius = rect.width() / (l_max + 1) as f32 / 2.0;
        for l in 0..=l_max {
            for m in 0..=l as i32 {
                let circle_pos = rect.left_top()
                    + egui::vec2(m as f32, l as f32)
                        * ((rect.width() - circle_radius * 2.0) / l_max as f32)
                    + egui::Vec2::splat(circle_radius);
                ui.painter().circle(
                    circle_pos,
                    circle_radius,
                    if l == self.l && m == self.m {
                        Color32::DARK_GREEN
                    } else if l == self.l && m == -self.m {
                        Color32::from_rgb(0, 0x64, 0x64)
                    } else {
                        Color32::DARK_GRAY
                    },
                    Stroke::NONE,
                );
            }
        }

        ui.put(
            Rect::from_min_max(rect.min + egui::vec2(rect.width() - 150.0, 4.0), rect.max),
            |ui: &mut Ui| {
                ui.horizontal_wrapped(|ui| {
                    let text_size = 36.0;
                    ui.spacing_mut().item_spacing *= 0.0;
                    ui.heading(RichText::new("Y").size(text_size));
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!(" {}", self.m)).size(text_size / 2.0));
                        ui.label(RichText::new(format!("{}", self.l)).size(text_size / 2.0));
                    });
                    ui.heading(RichText::new("(θ, φ)").size(text_size))
                })
                .inner
            },
        );
        ui.advance_cursor_after_rect(rect);
    }
}
