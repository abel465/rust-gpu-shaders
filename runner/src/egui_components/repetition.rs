use egui::{ComboBox, DragValue, Ui};
use glam::{UVec2, Vec2};
use strum::IntoEnumIterator;

#[cfg_attr(not(target_arch = "spirv"), derive(strum::EnumIter, strum::Display))]
#[derive(Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum RepetitionValue {
    None,
    Unlimited,
    Limited,
    Rectangular,
    Angular,
    Mirrored,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Repetition {
    pub current: RepetitionValue,
    pub unlimited: Vec2,
    pub mirrored: Vec2,
    pub limited: (Vec2, UVec2, UVec2),
    pub rectangular: (f32, UVec2),
    pub angular: (f32, u32),
}

impl Repetition {
    pub fn ui(&mut self, ui: &mut Ui) {
        use RepetitionValue::*;
        ComboBox::from_id_source("repetition")
            .selected_text(format!("{}", self.current))
            .show_ui(ui, |ui| {
                for repetition in RepetitionValue::iter() {
                    ui.selectable_value(&mut self.current, repetition, repetition.to_string());
                }
            });
        match self.current {
            None => {}
            Unlimited => {
                ui.horizontal(|ui| {
                    ui.label("Dim");
                    drag_value_dim(ui, &mut self.unlimited.x);
                    drag_value_dim(ui, &mut self.unlimited.y);
                });
            }
            Mirrored => {
                ui.horizontal(|ui| {
                    ui.label("Dim");
                    drag_value_dim(ui, &mut self.mirrored.x);
                    drag_value_dim(ui, &mut self.mirrored.y);
                });
            }
            Limited => {
                ui.horizontal(|ui| {
                    ui.label("N1");
                    ui.add(DragValue::new(&mut self.limited.1.x).clamp_range(0..=4));
                    ui.add(DragValue::new(&mut self.limited.1.y).clamp_range(0..=4));
                });
                ui.horizontal(|ui| {
                    ui.label("N2");
                    ui.add(DragValue::new(&mut self.limited.2.x).clamp_range(0..=4));
                    ui.add(DragValue::new(&mut self.limited.2.y).clamp_range(0..=4));
                });
                ui.horizontal(|ui| {
                    ui.label("Dim");
                    drag_value_dim(ui, &mut self.limited.0.x);
                    drag_value_dim(ui, &mut self.limited.0.y);
                });
            }
            Rectangular => {
                ui.horizontal(|ui| {
                    ui.label("N");
                    ui.add(DragValue::new(&mut self.rectangular.1.x).clamp_range(1..=4));
                    ui.add(DragValue::new(&mut self.rectangular.1.y).clamp_range(1..=4));
                });
                ui.horizontal(|ui| {
                    ui.label("Dim");
                    drag_value_dim(ui, &mut self.rectangular.0);
                });
            }
            Angular => {
                ui.horizontal(|ui| {
                    ui.label("N");
                    ui.add(DragValue::new(&mut self.angular.1).clamp_range(1..=10));
                });
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    drag_value_dim(ui, &mut self.angular.0);
                });
            }
        }
    }
}

fn drag_value_dim(ui: &mut Ui, value: &mut f32) {
    ui.add(DragValue::new(value).clamp_range(0.01..=1.0).speed(0.01));
}

impl Default for Repetition {
    fn default() -> Self {
        Self {
            current: RepetitionValue::None,
            unlimited: Vec2::splat(0.5),
            mirrored: Vec2::splat(0.5),
            limited: (Vec2::splat(0.5), UVec2::new(1, 0), UVec2::splat(1)),
            rectangular: (0.5, UVec2::new(3, 2)),
            angular: (0.3, 5),
        }
    }
}
