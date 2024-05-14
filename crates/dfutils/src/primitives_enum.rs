use crate::{primitives::*, sdf::Sdf};
use glam::Vec2;

#[cfg_attr(feature = "strum", derive(strum::EnumIter, strum::IntoStaticStr))]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
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
