#![cfg_attr(target_arch = "spirv", no_std)]

use crate::vec_functional::*;
use shared::{
    fast_optional::Optional_f32,
    push_constants::sdfs_3d::{sdf_shape, sdf_slice, Params, ShaderConstants, Shape},
    sdf_3d::{self as sdf, ops},
    *,
};
use spirv_std::glam::{vec3, Mat3, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[derive(PartialEq)]
#[repr(C)]
enum RayMarchResult {
    Divergent,
    Shape,
    DistanceTexture,
}

const COL_INSIDE: Vec3 = vec3(0.65, 0.85, 1.0);
const COL_OUTSIDE: Vec3 = vec3(0.9, 0.6, 0.3);
const YELLOW: Vec3 = vec3(1.0, 1.0, 0.0);

const MAX_STEPS: u32 = 100;
const MAX_DIST: f32 = 100.0;
const SURF_DIST: f32 = 0.0001;

fn sdf_ball(p: Vec3, cursor: Vec3, cursor_d: f32) -> f32 {
    sdf::sphere(p - cursor, cursor_d)
}

fn ray_march(
    ro: Vec3,
    rd: Vec3,
    shape: Shape,
    slice_z: f32,
    params: Params,
    cursor: Vec3,
    cursor_d: f32,
    mouse_pressed: bool,
    onion: Optional_f32,
) -> (f32, RayMarchResult) {
    let mut d0 = 0.0;
    let mut result = RayMarchResult::Divergent;

    for _ in 0..MAX_STEPS {
        let p = ro + rd * d0;
        let slice_d = sdf_slice(p, slice_z);
        let sliced_shape_d = ops::difference(sdf_shape(p, shape, params, onion), slice_d);
        let sliced_ball_d = ops::difference(sdf_ball(p, cursor, cursor_d), slice_d);
        let mut ds = sliced_shape_d;
        if mouse_pressed {
            ds = ds.min(sliced_ball_d)
        }
        d0 += ds;
        if ds < SURF_DIST {
            result = if mouse_pressed && ds == sliced_ball_d {
                RayMarchResult::DistanceTexture
            } else {
                RayMarchResult::Shape
            };
            break;
        }
        if d0 > MAX_DIST {
            break;
        }
    }

    (d0, result)
}

fn ray_march_distance_texture(
    ro: Vec3,
    rd: Vec3,
    slice_z: f32,
    cursor: Vec3,
    cursor_d: f32,
) -> f32 {
    let mut d0 = 0.0;

    for _ in 0..MAX_STEPS {
        let p = ro + rd * d0;
        let ds = ops::difference(sdf::sphere(p - cursor, cursor_d), sdf_slice(p, slice_z));
        d0 += ds;
        if ds < SURF_DIST || d0 > MAX_DIST {
            break;
        }
    }

    d0
}

fn get_d_to_shape_at_slice(
    ro: Vec3,
    rd: Vec3,
    shape: Shape,
    slice_z: f32,
    params: Params,
    onion: Optional_f32,
) -> f32 {
    let x = (slice_z - ro.z) / rd.z;
    if x < 0.0 {
        MAX_DIST
    } else {
        sdf_shape(ro + rd * x, shape, params, onion)
    }
}

fn get_d_to_cursor_at_slice(ro: Vec3, rd: Vec3, slice_z: f32, cursor: Vec3) -> f32 {
    let x = (slice_z - ro.z) / rd.z;
    if x < 0.0 {
        MAX_DIST
    } else {
        (ro + rd * x).distance(cursor)
    }
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let translate: Vec2 = constants.translate.into();
    let cursor: Vec3 = constants.cursor.into();

    let uv = from_pixels(frag_coord.xy(), constants.size);

    let rm = Mat3::from_rotation_y(translate.x).mul_mat3(&Mat3::from_rotation_x(translate.y));
    let ro = rm.mul_vec3(-Vec3::Z);
    let rd = rm.mul_vec3(uv.extend(1.0)).normalize();

    let slice_z = constants.slice_z;
    let mouse_pressed = constants.mouse_button_pressed & 1 != 0;
    let shape = Shape::from_u32(constants.shape);
    let onion = constants.onion;
    let slice_d = get_d_to_shape_at_slice(ro, rd, shape, slice_z, constants.params, onion);
    let cursor_d = sdf_shape(cursor, shape, constants.params, onion).abs();
    let cursor_d2 = sdf_shape(cursor, shape, constants.params, onion);
    let (d0, ray_march_result) = ray_march(
        ro,
        rd,
        shape,
        slice_z,
        constants.params,
        cursor,
        cursor_d,
        mouse_pressed,
        onion,
    );
    let d1 = if mouse_pressed {
        ray_march_distance_texture(ro, rd, slice_z, cursor, cursor_d)
    } else {
        MAX_DIST
    };
    let shape_col = Vec3::splat((ro + rd * d0).map(|x| (x * 50.0).sin().abs()).sum() * 0.3);

    let col = if d0 >= MAX_DIST {
        COL_OUTSIDE
    } else if d1 >= MAX_DIST {
        shape_col
    } else {
        let d_to_cursor = get_d_to_cursor_at_slice(ro, rd, slice_z, cursor);
        let sphere_surface_col = YELLOW
            * ((ro.z + rd.z * d1 - cursor.z) * 30.0 / cursor_d.sqrt())
                .sin()
                .abs();
        let sphere_intersection_col = YELLOW * (d_to_cursor * PI * 4.0 / cursor_d).sin().abs();
        let sphere_surface_only = !(mouse_pressed && d_to_cursor < cursor_d);
        sphere_surface_col
            .lerp(
                sphere_intersection_col,
                if sphere_surface_only {
                    0.0
                } else if ro.z < 0.0 {
                    1.0
                } else {
                    0.5
                },
            )
            .lerp(
                shape_col,
                if ray_march_result == RayMarchResult::DistanceTexture {
                    0.0
                } else {
                    0.5
                },
            )
    };

    let col = if (ray_march_result == RayMarchResult::DistanceTexture)
        || (d1 < MAX_DIST && cursor_d2 < 0.0)
        || (ray_march_result == RayMarchResult::Shape && ro.z > slice_z && slice_d > 0.0)
    {
        col
    } else if slice_d < 1.0 {
        let base = if slice_d < 0.0 { COL_INSIDE } else { col };
        let s = if slice_d < 0.0 && ro.z > slice_z {
            0.8
        } else {
            1.0
        };
        col.lerp(
            (base * (1.0 - (-6.0 * slice_d.abs()).exp())) * (0.8 + 0.2 * (150.0 * slice_d).cos()),
            s,
        )
        .lerp(Vec3::ONE, 1.0 - smoothstep(0.0, 0.005, slice_d.abs()))
    } else {
        col * 0.8
    };

    *output = col.extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    fullscreen_vs(vert_id, out_pos)
}
