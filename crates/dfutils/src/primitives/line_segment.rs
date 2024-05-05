use crate::sdf::Sdf;
use glam::{vec2, Vec2};

#[derive(Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub a: Vec2,
    pub b: Vec2,
}

impl core::fmt::Debug for LineSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Point")
            .field("a", &self.a.to_array())
            .field("b", &self.b.to_array())
            .finish()
    }
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
    fn signed_distance(&self, p: Vec2) -> f32 {
        let b = self.b - self.a;
        p.distance(self.a + b * ((p - self.a).dot(b) / b.length_squared()).clamp(0.0, 1.0))
    }

    fn distance(&self, p: Vec2) -> f32 {
        self.signed_distance(p)
    }
}
