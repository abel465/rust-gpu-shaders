use crate::sdf::Sdf;
use glam::Vec2;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct Torus {
    pub major_radius: f32,
    pub minor_radius: f32,
}

impl Torus {
    pub const fn new(major_radius: f32, minor_radius: f32) -> Self {
        Self {
            major_radius,
            minor_radius,
        }
    }
}

impl Default for Torus {
    fn default() -> Self {
        Self {
            major_radius: 0.2,
            minor_radius: 0.1,
        }
    }
}

impl Sdf for Torus {
    fn signed_distance(&self, p: Vec2) -> f32 {
        Disk::new(self.major_radius).distance(p) - self.minor_radius
    }
}
