use crate::sdf::Sdf;
use glam::Vec2;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plane {
    pub normal: Vec2,
}

impl Plane {
    pub const fn new(normal: Vec2) -> Self {
        Self { normal }
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self { normal: Vec2::Y }
    }
}

impl Sdf for Plane {
    type T = f32;
    fn signed_distance(&self, p: Vec2) -> f32 {
        self.normal.dot(p)
    }
}
