use crate::sdf::Sdf;
use glam::{vec2, Vec2};

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub a: Vec2,
    pub b: Vec2,
}

impl LineSegment {
    pub const fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }
}

impl Default for LineSegment {
    fn default() -> Self {
        Self {
            a: vec2(-0.2, -0.15),
            b: vec2(0.2, 0.15),
        }
    }
}

impl Sdf for LineSegment {
    type T = f32;
    fn signed_distance(&self, p: Vec2) -> f32 {
        let b = self.b - self.a;
        p.distance(self.a + b * ((p - self.a).dot(b) / b.length_squared()).clamp(0.0, 1.0))
    }

    fn distance(&self, p: Vec2) -> f32 {
        self.signed_distance(p)
    }
}
