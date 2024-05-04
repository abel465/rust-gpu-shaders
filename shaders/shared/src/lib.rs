#![cfg_attr(target_arch = "spirv", no_std)]
#![feature(variant_count)]

pub mod assert;
pub mod complex;
pub mod fast_optional;
pub mod functional;
pub mod interpreter;
pub mod numeric_integration;
pub mod push_constants;
pub mod random;
pub mod ray_intersection;
pub mod sdf_2d;
pub mod sdf_3d;
pub mod sdf_interpreter;
pub mod spherical_harmonics;
pub mod stack;
pub mod voronoi;

use push_constants::Size;
use spirv_std::glam::{vec2, Vec2, Vec4};

pub const SQRT_3: f32 = 1.7320508075688772;
pub use core::f32::consts::PI;

pub fn fullscreen_vs(vert_id: i32, out_pos: &mut Vec4) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *out_pos = pos.extend(0.0).extend(1.0);
}

pub fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}

pub fn from_pixels(Vec2 { x, y }: Vec2, Size { width, height }: Size) -> Vec2 {
    (vec2(x, -y) - 0.5 * vec2(width as f32, -(height as f32))) / height as f32
}

macro_rules! reduce {
    ($name:tt, $x:expr) => ( $x );
    ($name:tt, $x:expr, $($xs:expr),+) => {
        {
            $name($x, reduce!($name, $($xs),+))
        }
    };
}

pub(crate) use reduce;
