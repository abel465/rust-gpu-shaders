use crate::{primitives::*, sdf::Sdf};
use glam::Vec2;

pub enum Shape {
    Disk(Disk),
    Torus(Torus),
}

impl Sdf for Shape {
    fn signed_distance(&self, p: Vec2) -> f32 {
        match self {
            Shape::Disk(disk) => disk.signed_distance(p),
            Shape::Torus(torus) => torus.signed_distance(p),
        }
    }
}
