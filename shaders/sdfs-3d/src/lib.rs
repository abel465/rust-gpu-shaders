#![cfg_attr(target_arch = "spirv", no_std)]

use crate::functional::vec::*;
use shared::{
    push_constants::sdfs_3d::{Params, ShaderConstants, Shape, MAX_NUM_POINTS},
    ray_intersection::{ray_intersect_capsule, ray_intersect_hemisphere, ray_intersect_sphere},
    sdf_3d::{self as sdf, ops},
    *,
};
use spirv_std::glam::{vec3, Mat3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

const COL_INSIDE: Vec3 = vec3(0.65, 0.85, 1.0);
const COL_OUTSIDE: Vec3 = vec3(0.9, 0.6, 0.3);
const YELLOW: Vec3 = vec3(1.0, 1.0, 0.0);

const MAX_STEPS: u32 = 512;
const MAX_DIST: f32 = 100.0;
const SURF_DIST: f32 = 0.0002;

pub fn sdf_shape(mut p: Vec3, shape: Shape, params: Params) -> f32 {
    use Shape::*;

    let dim = vec3(params.dims[0], params.dims[1], params.dims[2]);
    let dim2 = vec3(params.dims[3], params.dims[4], params.dims[5]);
    let p0 = params.ps[0].into();
    let p1 = params.ps[1].into();
    let orientation = Vec3::Y;

    if params.repeat[0].has_value() {
        p = ops::fast_repeat::repeat_x(p, params.repeat[0].value)
    }

    if params.repeat[1].has_value() {
        p = ops::fast_repeat::repeat_y(p, params.repeat[1].value)
    }

    if params.repeat[2].has_value() {
        p = ops::fast_repeat::repeat_z(p, params.repeat[2].value)
    }

    let mut d = match shape {
        Sphere => sdf::sphere(p, dim.x),
        Cuboid => sdf::cuboid(p, dim),
        CuboidFrame => sdf::cuboid_frame(p, dim, dim2),
        CuboidFrameRadial => sdf::cuboid_frame_radial(p, dim, dim2.x),
        Capsule => sdf::capsule(p, p0, p1, dim.x),
        Cylinder => sdf::cylinder(p, p0, p1, dim.x),
        Torus => sdf::torus(p, dim.xy(), orientation),
        Disk => sdf::disk(p, dim.xy(), orientation),
        Plane => sdf::plane(p, orientation),
    };

    if params.pad.has_value() {
        d = ops::pad(d, params.pad.value)
    }

    if params.onion.has_value() {
        d = ops::onion(d, params.onion.value)
    }

    d
}

pub fn sdf_slice(p: Vec3, slice_z: f32) -> f32 {
    sdf::plane(p - slice_z * Vec3::Z, Vec3::Z)
}

fn ray_march(ro: Vec3, rd: Vec3, shape: Shape, slice_z: f32, params: Params) -> f32 {
    let mut d0 = 0.0;

    for _ in 0..MAX_STEPS {
        let p = ro + rd * d0;
        let ds = ops::difference(sdf_shape(p, shape, params), sdf_slice(p, slice_z));
        d0 += ds;
        if ds < SURF_DIST || d0 > MAX_DIST {
            break;
        }
    }

    d0
}

fn get_d_to_shape_at_slice(ro: Vec3, rd: Vec3, shape: Shape, slice_z: f32, params: Params) -> f32 {
    let x = (slice_z - ro.z) / rd.z;
    if x < 0.0 {
        MAX_DIST
    } else {
        sdf_shape(ro + rd * x, shape, params)
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
    let slice_d = get_d_to_shape_at_slice(ro, rd, shape, slice_z, constants.params);
    let cursor_d = sdf_shape(cursor, shape, constants.params);
    let d0 = ray_march(ro, rd, shape, slice_z, constants.params);
    let hemisphere_d = if mouse_pressed {
        ray_intersect_hemisphere(ro, rd, cursor, Vec3::Z, cursor_d.abs())
    } else {
        0.0
    };
    let shape_col =
        Vec3::splat((ro + rd * d0).map(|x| (x * 50.0).sin().abs()).sum() * 0.3) / d0.max(1.0);
    let d_to_cursor = get_d_to_cursor_at_slice(ro, rd, slice_z, cursor);
    let mut col = if hemisphere_d <= 0.0 && d0 >= MAX_DIST {
        COL_OUTSIDE
    } else if hemisphere_d <= 0.0 {
        shape_col
    } else {
        let sphere_surface_col = YELLOW
            * ((ro + rd * hemisphere_d - cursor).z * 30.0 / cursor_d.abs().sqrt())
                .sin()
                .abs();
        let sphere_intersection_col = YELLOW * (d_to_cursor * PI * 4.0 / cursor_d).sin().abs();
        let sphere_surface_only = !(d_to_cursor < cursor_d.abs());
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
                if (hemisphere_d < d0 && hemisphere_d > 0.0) || (slice_d < 0.0 && ro.z < 0.0) {
                    0.0
                } else {
                    0.5
                },
            )
    };

    if !(d_to_cursor < cursor_d.abs()
        || (d0 < MAX_DIST && ro.z > slice_z && slice_d > 0.0)
        || slice_d >= MAX_DIST)
    {
        let base = if slice_d < 0.0 && hemisphere_d <= 0.0 {
            COL_INSIDE
        } else {
            col
        };
        let s = if slice_d < 0.0 && ro.z > slice_z {
            0.8
        } else {
            1.0
        };
        let phase = if slice_d.abs() < 1.0 { 0.0 } else { PI };
        let angle = if slice_d.abs() < 1.0 {
            slice_d
        } else {
            1.0 / slice_d
        };
        col = col
            .lerp(
                (base * (1.0 - (-40.0 * slice_d.abs()).exp()))
                    * (0.8 + 0.2 * (300.0 * angle + phase).cos()),
                s,
            )
            .lerp(Vec3::ONE, 1.0 - smoothstep(0.0, 0.005, slice_d.abs()))
    }

    if mouse_pressed {
        let der = {
            let e = 0.01;
            vec3(
                sdf_shape(cursor - e * Vec3::X, shape, constants.params)
                    - sdf_shape(cursor + e * Vec3::X, shape, constants.params),
                sdf_shape(cursor - e * Vec3::Y, shape, constants.params)
                    - sdf_shape(cursor + e * Vec3::Y, shape, constants.params),
                sdf_shape(cursor - e * Vec3::Z, shape, constants.params)
                    - sdf_shape(cursor + e * Vec3::Z, shape, constants.params),
            ) / (2.0 * e)
        };
        let r = (cursor_d.abs() / 8.0).min(0.01);
        let pa = cursor;
        let pb = cursor + der.normalize_or_zero() * cursor_d;
        let d = ray_intersect_capsule(ro, rd, pa, pb, r);
        if d > 0.0 {
            let p = ro + rd * d;
            let c = vec3(1.0, 0.5, 0.0) * (p.distance(cursor) * PI * 4.0 / (cursor_d)).cos().abs();
            col = if d < hemisphere_d {
                c
            } else {
                col.lerp(c, 0.7)
            };
        }
    }

    for i in 0..MAX_NUM_POINTS {
        let p: Vec3 = constants.params.ps[i].into();
        if ray_intersect_sphere(ro, rd, p, 0.01) {
            col = col.lerp(Vec3::ONE, 0.2);
        }
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
