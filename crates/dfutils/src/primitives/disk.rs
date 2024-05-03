use crate::sdf::Sdf;
use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disk {
    pub radius: f32,
}

impl Disk {
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Default for Disk {
    fn default() -> Self {
        Self { radius: 0.3 }
    }
}

impl Sdf for Disk {
    fn signed_distance(&self, p: Vec2) -> f32 {
        p.length() - self.radius
    }
}
