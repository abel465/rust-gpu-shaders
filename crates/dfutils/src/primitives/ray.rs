use crate::sdf::Sdf;
use glam::Vec2;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Ray {
    pub direction: Vec2,
}

impl Ray {
    pub const fn new(direction: Vec2) -> Self {
        Self { direction }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self { direction: Vec2::X }
    }
}

impl Sdf for Ray {
    type T = f32;
    fn signed_distance(&self, p: Vec2) -> f32 {
        p.distance(self.direction * p.dot(self.direction).max(0.0))
    }

    fn distance(&self, p: Vec2) -> f32 {
        self.signed_distance(p)
    }
}
