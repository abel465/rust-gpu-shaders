#![cfg_attr(target_arch = "spirv", no_std)]

use push_constants::sierpinski_triangle::ShaderConstants;
use sdf_2d::fractal::sierpinski_triangle;
use shared::*;
use spirv_std::glam::{vec3, Vec2, Vec4, Vec4Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let uv = constants.zoom * from_pixels(frag_coord.xy(), constants.size);
    let dim: Vec2 = constants.dim.into();

    let d = sierpinski_triangle(uv - dim, 0.5 / SQRT_3, 22);
    let thickness = constants.zoom / constants.size.height as f32;
    let col = vec3(0.9, 0.6, 0.4) * smoothstep(thickness, 0.0, d);

    *output = col.extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    fullscreen_vs(vert_id, out_pos)
}
