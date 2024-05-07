use crate::sdf::Sdf;
use glam::{vec2, Vec2};

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            width: 0.5,
            height: 0.3,
        }
    }
}

impl Sdf for Rectangle {
    fn signed_distance(&self, p: Vec2) -> f32 {
        let p = p.abs() - vec2(self.width, self.height) * 0.5;
        p.max(Vec2::ZERO).length() + p.min(Vec2::ZERO).max_element()
    }

    fn distance(&self, p: Vec2) -> f32 {
        let p = p.abs() - vec2(self.width, self.height) * 0.5;
        p.max(Vec2::ZERO).length() - p.min(Vec2::ZERO).max_element()
    }
}
