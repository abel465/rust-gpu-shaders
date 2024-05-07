use crate::primitives::*;

#[cfg_attr(feature = "strum", derive(strum::EnumIter, strum::IntoStaticStr))]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
#[enum_dispatch::enum_dispatch(Sdf)]
pub enum Shape {
    Disk(Disk),
    Torus(Torus),
    Rectangle(Rectangle),
    Cross(Cross),
    LineSegment(LineSegment),
    Plane(Plane),
}
