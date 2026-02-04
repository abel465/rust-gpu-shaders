use crate::{primitives::*, sdf::Sdf};
use glam::Vec2;

#[cfg_attr(feature = "strum", derive(strum::EnumIter, strum::IntoStaticStr))]
#[derive(Clone, Copy, PartialEq, Debug)]
#[enum_delegate::implement(Sdf)]
pub enum Shape {
    Disk(Disk),
    Torus(Torus),
    Rectangle(Rectangle),
    Cross(Cross),
    Plane(Plane),
    Ray(Ray),
    LineSegment(LineSegment),
}
