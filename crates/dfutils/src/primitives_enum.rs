use crate::{primitives::*, sdf::Sdf};
use glam::Vec2;

#[cfg_attr(feature = "strum", derive(strum::EnumIter, strum::IntoStaticStr))]
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Disk(Disk),
    Torus(Torus),
    Rectangle(Rectangle),
    Cross(Cross),
    LineSegment(LineSegment),
}

impl Sdf for Shape {
    fn signed_distance(&self, p: Vec2) -> f32 {
        use Shape::*;
        match self {
            Disk(disk) => disk.signed_distance(p),
            Torus(torus) => torus.signed_distance(p),
            Rectangle(rectangle) => rectangle.signed_distance(p),
            Cross(cross) => cross.signed_distance(p),
            LineSegment(line_segment) => line_segment.signed_distance(p),
        }
    }

    fn distance(&self, p: Vec2) -> f32 {
        use Shape::*;
        match self {
            Disk(disk) => disk.distance(p),
            Torus(torus) => torus.distance(p),
            Rectangle(rectangle) => rectangle.distance(p),
            Cross(cross) => cross.distance(p),
            LineSegment(line_segment) => line_segment.distance(p),
        }
    }
}
