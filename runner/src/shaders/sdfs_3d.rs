use crate::egui_components::enabled_number::EnabledNumber;
use crate::window::UserEvent;
use bytemuck::Zeroable;
use egui::{Context, CursorIcon};
use glam::{vec2, Mat3, Vec2, Vec3};
use shared::{
    from_pixels,
    push_constants::sdfs_3d::{Params, ShaderConstants, Shape},
    ray_intersection::ray_intersect_sphere,
};
use std::{f32::consts::PI, time::Instant};
use strum::IntoEnumIterator;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::EventLoopProxy,
};

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    mouse_button_pressed: u32,
    cursor: Vec2,
    prev_cursor: Vec2,
    can_drag: Option<usize>,
    drag_point: Option<usize>,
    shape: Shape,
    params: Vec<Params>,
    shader_constants: ShaderConstants,
    camera: Vec2,
    slice_z: f32,
    onion: EnabledNumber<f32>,
    pad: EnabledNumber<f32>,
    repeat_x: EnabledNumber<f32>,
    repeat_y: EnabledNumber<f32>,
    repeat_z: EnabledNumber<f32>,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            mouse_button_pressed: 0,
            can_drag: None,
            drag_point: None,
            shape: Shape::Sphere,
            params: Shape::iter().map(|shape| shape.default_params()).collect(),
            shader_constants: ShaderConstants::zeroed(),
            camera: vec2(0.2, 0.7),
            slice_z: 0.0,
            onion: EnabledNumber::new(0.05, false),
            pad: EnabledNumber::new(0.05, false),
            repeat_x: EnabledNumber::new(0.5, false),
            repeat_y: EnabledNumber::new(0.5, false),
            repeat_z: EnabledNumber::new(0.5, false),
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match button {
            MouseButton::Left => match state {
                ElementState::Pressed => {
                    self.drag_point = self.can_drag;
                    self.mouse_button_pressed |= 1;
                }
                ElementState::Released => {
                    self.drag_point = None;
                    self.mouse_button_pressed &= !1;
                }
            },
            MouseButton::Right => match state {
                ElementState::Pressed => {
                    self.mouse_button_pressed |= 1 << 2;
                }
                ElementState::Released => {
                    self.mouse_button_pressed &= !(1 << 2);
                }
            },
            _ => {}
        }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
        let num_points = self.shape.default_points().len();
        let translate = self.camera;
        let rm = Mat3::from_rotation_y(translate.x).mul_mat3(&Mat3::from_rotation_x(translate.y));
        if let Some(i) = self.drag_point {
            let pc = from_pixels(self.prev_cursor, self.size.into());
            let cc = from_pixels(self.cursor, self.size.into());
            let t = rm.mul_vec3((cc - pc).extend(0.0));
            let p: Vec3 = self.params[self.shape as usize].ps[i].into();
            self.params[self.shape as usize].ps[i] = (p + t).into();
        } else if num_points > 0 {
            let ro = rm.mul_vec3(-Vec3::Z);
            let rd = rm
                .mul_vec3(from_pixels(self.cursor, self.size.into()).extend(1.0))
                .normalize();
            self.can_drag = self.params[self.shape as usize].ps[0..num_points as usize]
                .iter()
                .position(|p| ray_intersect_sphere(ro, rd, (*p).into(), 0.01));
        }
        if self.mouse_button_pressed & (1 << 2) != 0 {
            self.camera += PI * (self.cursor - self.prev_cursor) / self.size.height as f32;
        }
        self.prev_cursor = self.cursor;
    }

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.slice_z += match delta {
            MouseScrollDelta::LineDelta(_, y) => 0.01 * y,
            MouseScrollDelta::PixelDelta(p) => {
                0.01 * (1.0 + p.y.abs() as f32).ln() * p.y.signum() as f32
            }
        };
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    fn update(&mut self) {
        let cursor_3d_pos = if self.mouse_button_pressed & 1 == 1 {
            self.get_cursor_slice_pos()
        } else {
            Vec3::MAX
        };
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            cursor: cursor_3d_pos.into(),
            mouse_button_pressed: if self.drag_point.is_some() || cursor_3d_pos == Vec3::MAX {
                self.mouse_button_pressed & !1
            } else {
                self.mouse_button_pressed
            },
            slice_z: self.slice_z,
            translate: self.camera.into(),
            shape: self.shape as u32,
            params: self.params(),
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, ctx: &Context, ui: &mut egui::Ui, _: &EventLoopProxy<UserEvent>) {
        ctx.set_cursor_icon(if self.drag_point.is_some() {
            CursorIcon::Grabbing
        } else if self.can_drag.is_some() && self.mouse_button_pressed == 0 {
            CursorIcon::Grab
        } else {
            CursorIcon::Default
        });
        for shape in Shape::iter() {
            ui.radio_value(&mut self.shape, shape, shape.to_string());
        }
        ui.separator();
        self.pad.ui(ui, "Pad", 0.0..=0.1, 0.01);
        self.onion.ui(ui, "Onion", 0.0..=0.1, 0.01);
        self.repeat_x.ui(ui, "Repeat X", 0.01..=1.0, 0.01);
        self.repeat_y.ui(ui, "Repeat Y", 0.01..=1.0, 0.01);
        self.repeat_z.ui(ui, "Repeat Z", 0.01..=1.0, 0.01);
        let params = &mut self.params[self.shape as usize];
        let labels = self.shape.labels();
        if labels.len() > 0 {
            ui.separator();
        }
        for i in 0..labels.len() {
            let ranges = self.shape.dim_range();
            let range = ranges[i].clone();
            let speed = (range.end() - range.start()) * 0.02;
            ui.horizontal(|ui| {
                ui.label(labels[i as usize]);
                ui.add(
                    egui::DragValue::new(&mut params.dims[i as usize])
                        .clamp_range(range)
                        .speed(speed),
                );
            });
        }
    }
}

impl Controller {
    fn params(&self) -> Params {
        Params {
            onion: self.onion.into(),
            pad: self.pad.into(),
            repeat: [
                self.repeat_x.into(),
                self.repeat_y.into(),
                self.repeat_z.into(),
            ],
            ..self.params[self.shape as usize]
        }
    }

    fn get_cursor_slice_pos(&self) -> Vec3 {
        let cursor = from_pixels(self.cursor, self.size.into());
        let translate = self.camera;
        let rm = Mat3::from_rotation_y(translate.x).mul_mat3(&Mat3::from_rotation_x(translate.y));
        let ro = rm.mul_vec3(-Vec3::Z);
        let rd = rm.mul_vec3(cursor.extend(1.0)).normalize();
        let x = (self.slice_z - ro.z) / rd.z;
        if x < 0.0 {
            Vec3::MAX
        } else {
            ro + rd * x
        }
    }
}
