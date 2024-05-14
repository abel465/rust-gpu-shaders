#![cfg_attr(target_arch = "spirv", no_std)]

use push_constants::sdfs_2d::{ShaderConstants, MAX_NUM_POINTS};
use sdf::grid::SdfGrid;
use sdf_2d as sdf;
use shared::*;
use spirv_std::glam::{vec3, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

fn sdf(p: Vec2, grid: &SdfGrid, smooth: bool) -> f32 {
    let p = grid.clamp(p);
    if smooth {
        grid.dist_smooth(p)
    } else {
        grid.dist(p)
    }
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] grid: &SdfGrid,
    output: &mut Vec4,
) {
    let uv = from_pixels(frag_coord.xy(), constants.size);
    let cursor = constants.cursor.into();
    let smooth: bool = constants.smooth.into();

    let d = sdf(uv, grid, smooth);
    let mut col = if d < 0.0 {
        vec3(0.65, 0.85, 1.0)
    } else {
        vec3(0.9, 0.6, 0.3)
    };
    col *= 1.0 - (-50.0 * d.abs()).exp();
    col *= 0.8 + 0.2 * (300.0 * d).cos();

    if constants.mouse_button_pressed & 1 != 0 {
        let d = sdf(cursor, grid, smooth);
        let der: Vec2 = constants.derivative_at_cursor.into();
        let p = uv - cursor;
        col = col.lerp(
            vec3(1.0, 1.0, 0.0),
            smoothstep(
                0.001,
                0.0,
                sdf::disk(p, 0.008)
                    .min(sdf::disk(p, d.abs()).abs())
                    .min(sdf::finite_ray(
                        p,
                        der.normalize_or_zero() * d.signum(),
                        d.abs(),
                    ))
                    - 0.0025,
            ),
        );
    }

    for i in 0..MAX_NUM_POINTS {
        let p: Vec2 = constants.points[i].into();
        col = col.lerp(Vec3::ONE, smoothstep(0.008, 0.0, sdf::disk(uv - p, 0.002)))
    }

    *output = col.powf(2.2).extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    fullscreen_vs(vert_id, out_pos)
}
