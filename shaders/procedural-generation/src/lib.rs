#![cfg_attr(target_arch = "spirv", no_std)]

use crate::functional::vec::*;
use push_constants::procedural_generation::ShaderConstants;
use sdf_3d as sdf;
use shared::*;
use spirv_std::glam::*;
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

const MAX_STEPS: u32 = 200;
const MAX_DIST: f32 = 100.0;
const SURF_DIST: f32 = 0.0001;

fn sdf(p: Vec3) -> f32 {
    sdf::plane(p - vec3(0.0, -0.3, 0.0), Vec3::Y)
}

fn ray_march(ro: Vec3, rd: Vec3) -> f32 {
    let mut d0 = 0.0;

    for _ in 0..MAX_STEPS {
        let p = ro + rd * d0;
        let ds = sdf(p);
        let ds = sd_fbm(p, ds);
        d0 += ds;
        if ds < SURF_DIST || d0 > MAX_DIST {
            break;
        }
    }

    d0
}
fn smin(d1: f32, d2: f32, k: f32) -> f32 {
    let h = saturate(0.5 + 0.5 * (d2 - d1) / k);
    mix(d2, d1, h) - k * h * (1.0 - h)
}

fn smax(d1: f32, d2: f32, k: f32) -> f32 {
    let h = saturate(0.5 - 0.5 * (d2 - d1) / k);
    mix(d2, d1, h) + k * h * (1.0 - h)
}

fn sph(i: Vec3, f: Vec3, c: Vec3) -> f32 {
    // random radius at grid vertex i+c
    let rad = 0.5 * random::random31(i + c);
    // distance to sphere at grid vertex i+c
    f.distance(c) - rad
}

fn sd_base(p: Vec3) -> f32 {
    let i = p.floor();
    let f = p.fract();

    // Distance to the 8 corners spheres

    sph(i, f, Vec3::ZERO)
        .min(sph(i, f, Vec3::Z))
        .min(sph(i, f, Vec3::Y))
        .min(sph(i, f, vec3(0.0, 1.0, 1.0)))
        .min(sph(i, f, Vec3::X))
        .min(sph(i, f, vec3(1.0, 0.0, 1.0)))
        .min(sph(i, f, vec3(1.0, 1.0, 0.0)))
        .min(sph(i, f, Vec3::ONE))
}

fn sd_fbm(mut p: Vec3, mut d: f32) -> f32 {
    let mut s = 1.0;
    const M: Mat3 =
        Mat3::from_cols_array(&[0.00, 1.60, 1.20, -1.60, 0.72, -0.96, -1.20, -0.96, 1.28]);

    for _ in 0..4 {
        // Evaluate new octave
        let n = s * sd_base(p);

        // Add
        let n = smax(n, d - 0.1 * s, 0.3 * s);
        d = smin(n, d, 0.3 * s);

        // Prepare next octave
        p = M * p;
        s *= 0.5;
    }
    d
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let uv = from_pixels(frag_coord.xy(), constants.size);
    let t = constants.time;
    let p = vec3(t.sin(), 0.0, t);
    let ds = sd_fbm(p, sdf(p));
    let ro = vec3(p.x, ds * 0.5, p.z);
    let rd = uv.extend(-1.0);
    let d = ray_march(ro, rd);
    let p = ro + rd * d;
    let col = if d >= MAX_DIST {
        Vec3::ZERO
    } else {
        if p.y < -0.29 {
            let c = ((p.y - 0.3) * 40.0).cos() / d.max(1.0);
            Vec3::splat(c)
        } else {
            let x = p.map(|x| (x * 50.0).sin()).sum().sin().abs() / d.max(1.0);
            let y = 1.0 - (p.y * 20.0).cos();
            let x = mix(x, y, 0.4) / d.max(1.0);
            vec3(x, x * x, x * x)
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
