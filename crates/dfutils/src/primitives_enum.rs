use crate::primitives::*;

#[cfg_attr(feature = "strum", derive(strum::EnumIter, strum::IntoStaticStr))]
#[derive(Clone, Copy, Debug)]
#[enum_dispatch::enum_dispatch(Sdf)]
pub enum Shape {
    Disk(Disk),
    Torus(Torus),
    Rectangle(Rectangle),
    Cross(Cross),
    LineSegment(LineSegment),
}
