#![cfg_attr(target_arch = "spirv", no_std)]

use core::f32::consts::FRAC_1_SQRT_2;
use push_constants::spherical_harmonics::{ShaderConstants, Variant};
use shared::{ray_intersection::ray_intersect_box_frame, spherical_harmonics::*, *};
use spirv_std::glam::{vec2, vec3, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let uv = from_pixels(frag_coord.xy(), constants.size);
    let rot: Quat = constants.rot.into();
    let r = 0.3 / constants.zoom;
    let z2 = r * r - uv.length_squared();

    let col = if z2 > 0.0 {
        let pos = rot * uv.extend(z2.sqrt());
        let (_, theta, phi) = to_spherical(pos);
        let m = constants.m;
        let l = constants.l;
        match Variant::from_u32(constants.variant) {
            Variant::Real => {
                let x = real_spherical_harmonic(m, l, theta, phi, constants.time);
                vec3(x, 0.0, -x)
            }
            Variant::Complex => {
                let z = spherical_harmonic(m, l, theta, phi, constants.time);
                vec3(
                    z.dot(Vec2::X),
                    z.dot(vec2(-FRAC_1_SQRT_2, FRAC_1_SQRT_2)),
                    z.dot(Vec2::splat(-FRAC_1_SQRT_2)),
                )
            }
        }
    } else {
        let ro = rot * uv.extend(-constants.zoom);
        let rd = rot * Vec2::ZERO.extend(1.0);
        if ray_intersect_box_frame(ro, rd, vec2(r, 0.002 / constants.zoom)) {
            vec3(0.1, 0.1, 0.08)
        } else {
            Vec3::ZERO
        }
    };

    *output = col.powf(2.2).extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    fullscreen_vs(vert_id, out_pos)
}
