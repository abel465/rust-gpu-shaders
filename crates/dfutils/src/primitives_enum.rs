use crate::{primitives::*, sdf::Sdf};
use glam::Vec2;

#[cfg_attr(feature = "strum", derive(strum::EnumIter, strum::IntoStaticStr))]
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Disk(Disk),
    Torus(Torus),
    Rectangle(Rectangle),
}

impl Sdf for Shape {
    fn signed_distance(&self, p: Vec2) -> f32 {
        match self {
            Shape::Disk(disk) => disk.signed_distance(p),
            Shape::Torus(torus) => torus.signed_distance(p),
            Shape::Rectangle(rectangle) => rectangle.signed_distance(p),
        }
    }
}
