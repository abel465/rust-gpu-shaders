use super::{Size, Vec2, Vec3};
use crate::fast_optional::Optional_f32;
use bytemuck::{Pod, Zeroable};

#[cfg_attr(not(target_arch = "spirv"), derive(strum::EnumIter, strum::Display))]
#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum Shape {
    Sphere,
    Cuboid,
    CuboidFrame,
    CuboidFrameRadial,
    Capsule,
    Cylinder,
    Torus,
    Disk,
    Plane,
}

impl Shape {
    pub fn from_u32(x: u32) -> Self {
        if x >= core::mem::variant_count::<Shape>() as u32 {
            Shape::Sphere
        } else {
            unsafe { core::mem::transmute(x) }
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Shape {
    pub fn labels(self) -> &'static [&'static str] {
        use Shape::*;
        const R: &'static str = "Radius";
        const W: &'static str = "Width";
        const H: &'static str = "Height";
        const L: &'static str = "Length";
        match self {
            Sphere | Capsule | Cylinder => &[R],
            Cuboid => &[W, H, L],
            CuboidFrame => &[W, H, L, "Inner Width", "Inner Height", "Inner Length"],
            CuboidFrameRadial => &[W, H, L, R],
            Torus | Disk => &["Major Radius", "Minor Radius"],
            Plane => &[],
        }
    }

    pub fn dim_range(&self) -> &[core::ops::RangeInclusive<f32>] {
        use Shape::*;
        match self {
            Sphere => &[0.0..=0.5],
            Cuboid => &[0.0..=0.5, 0.0..=0.5, 0.0..=0.5],
            CuboidFrame => &[
                0.0..=0.5,
                0.0..=0.5,
                0.0..=0.5,
                0.0..=0.1,
                0.0..=0.1,
                0.0..=0.1,
            ],
            CuboidFrameRadial => &[0.0..=0.5, 0.0..=0.5, 0.0..=0.5, 0.0..=0.1],
            Capsule => &[0.0..=0.5],
            Cylinder => &[0.0..=0.5],
            Torus => &[0.0..=0.5, 0.0..=0.5],
            Disk => &[0.0..=0.5, 0.0..=0.5],
            Plane => &[],
        }
    }

    pub fn default_dims(&self) -> &[f32] {
        use Shape::*;
        match self {
            Sphere => &[0.2],
            Cuboid => &[0.2, 0.15, 0.2],
            CuboidFrame => &[0.2, 0.15, 0.2, 0.04, 0.04, 0.04],
            CuboidFrameRadial => &[0.2, 0.15, 0.2, 0.02],
            Capsule | Cylinder => &[0.2],
            Torus => &[0.2, 0.1],
            Disk => &[0.2, 0.02],
            Plane => &[],
        }
    }

    pub fn default_points(&self) -> &[[f32; 3]] {
        use Shape::*;
        match self {
            Capsule => &[[0.0, 0.0, -0.2], [0.0, 0.0, 0.2]],
            Cylinder => &[[0.0, 0.0, -0.3], [0.0, 0.0, 0.3]],
            _ => &[],
        }
    }

    pub fn default_params(&self) -> Params {
        let default_ps = self.default_points();
        let mut ps = [[1e10; 3]; 2];
        for i in 0..default_ps.len() {
            ps[i] = default_ps[i];
        }

        let default_dims = self.default_dims();
        let mut dims = [0.0; 6];
        for i in 0..default_dims.len() {
            dims[i] = default_dims[i];
        }

        Params {
            dims,
            ps,
            onion: Optional_f32::NONE,
            pad: Optional_f32::NONE,
            repeat: [Optional_f32::NONE; 3],
        }
    }
}

pub const MAX_NUM_POINTS: usize = 2;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Params {
    pub dims: [f32; 6],
    pub ps: [[f32; 3]; MAX_NUM_POINTS],
    pub onion: Optional_f32,
    pub pad: Optional_f32,
    pub repeat: [Optional_f32; 3],
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub size: Size,
    pub time: f32,
    pub cursor: Vec3,
    pub slice_z: f32,
    pub translate: Vec2,
    /// Bit mask of the pressed buttons (0 = Left, 1 = Middle, 2 = Right).
    pub mouse_button_pressed: u32,
    pub shape: u32,
    pub params: Params,
}
