//! Module containing 2d signed distance primitives.
//! Many are adapted from https://iquilezles.org/articles/distfunctions2d/ (Inigo Quilez)
//!

use crate::{
    functional::{tuple::*, vec::*},
    SQRT_3,
};
use core::f32::consts::PI;
use spirv_std::glam::{vec2, BVec3, Vec2, Vec2Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

pub mod fractal;
pub mod grid;
pub mod ops;
pub mod specialization;

pub fn disk(p: Vec2, r: f32) -> f32 {
    p.length() - r
}

pub fn rectangle(p: Vec2, dim: Vec2) -> f32 {
    let v = p.abs() - dim * 0.5;
    v.max(Vec2::ZERO).length() + v.max_element().min(0.0)
}

// `n` must be normalized
pub fn plane(p: Vec2, n: Vec2) -> f32 {
    p.dot(n)
}

// `n` must be normalized
pub fn line(p: Vec2, n: Vec2) -> f32 {
    plane(p, n).abs()
}

// `d` must be normalized
pub fn ray(p: Vec2, d: Vec2) -> f32 {
    p.distance(d * p.dot(d).max(0.0))
}

// `d` must be normalized
pub fn finite_ray(p: Vec2, d: Vec2, m: f32) -> f32 {
    p.distance(d * p.dot(d).clamp(0.0, m))
}

pub fn line_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    p.distance(a + (p - a).project_onto_segment(b - a))
}

pub fn capsule(p: Vec2, a: Vec2, b: Vec2, r: f32) -> f32 {
    line_segment(p, a, b) - r
}

pub fn torus(p: Vec2, r: Vec2) -> f32 {
    (p.length() - r.x).abs() - r.y
}

pub fn equilateral_triangle(mut p: Vec2, mut r: f32) -> f32 {
    const COS_FRAC_PI_6: f32 = 0.866025404;
    const K: f32 = SQRT_3;
    r *= COS_FRAC_PI_6;
    p = vec2(p.x.abs() - r, p.y + r / K);
    if p.x + K * p.y > 0.0 {
        p = vec2(p.x - K * p.y, -K * p.x - p.y) * 0.5;
    }
    p.x -= p.x.clamp(-2.0 * r, 0.0);
    -p.length() * p.y.signum()
}

pub fn isosceles_triangle(mut p: Vec2, mut dim: Vec2) -> f32 {
    p = vec2(p.x.abs(), dim.y - p.y);
    dim.x *= 0.5;
    let a = p.reject_from_segment(dim);
    let b = p - vec2(p.x.min(dim.x), dim.y);
    let s = dim.y.signum();
    (a, b).min_length() * (s * p.perp_dot(dim)).max(s * (p.y - dim.y)).signum()
}

pub fn triangle(p: Vec2, p0: Vec2, p1: Vec2, p2: Vec2) -> f32 {
    let e = (p1 - p0, p2 - p1, p0 - p2);
    let w = (p - p0, p - p1, p - p2);
    let ew = e.zip(w);
    let sgn = {
        let s = e.0.perp_dot(e.2).signum();
        -ew.map(|(e, w)| s * w.perp_dot(e)).min_element().signum()
    };
    sgn * ew
        .map(|(e, w)| w.reject_from_segment(e))
        .prepend(p - p0)
        .min_length()
}

pub fn polygon<const N: usize>(p: Vec2, ps: [Vec2; N]) -> f32 {
    let mut d = (p - ps[0]).length_squared();
    let mut s = 1.0;
    let mut j = N - 1;
    for i in 0..N {
        let e = ps[j] - ps[i];
        let w = p - ps[i];
        d = d.min(w.reject_from_segment(e).length_squared());
        let c = BVec3::new(p.y >= ps[i].y, p.y < ps[j].y, e.perp_dot(w) > 0.0);
        if c.all() || !c.any() {
            s = -s;
        }
        j = i;
    }
    s * d.sqrt()
}

pub fn hexagon(mut p: Vec2, r: f32) -> f32 {
    const COS_FRAC_PI_6: f32 = 0.866025404;
    const TAN_FRAC_PI_6: f32 = 0.577350269;
    const K: Vec2 = vec2(-COS_FRAC_PI_6, 0.5);
    p = p.abs();
    p -= 2.0 * K * p.dot(K).min(0.0);
    p -= vec2(p.x.clamp(-TAN_FRAC_PI_6 * r, TAN_FRAC_PI_6 * r), r);
    p.length() * p.y.signum()
}

pub fn pentagon(mut p: Vec2, r: f32) -> f32 {
    const SIN_FRAC_PI_5: f32 = 0.809016994;
    const COS_FRAC_PI_5: f32 = 0.587785252;
    const TAN_FRAC_PI_5: f32 = 0.726542528;
    const K1: Vec2 = vec2(-SIN_FRAC_PI_5, COS_FRAC_PI_5);
    const K2: Vec2 = vec2(SIN_FRAC_PI_5, COS_FRAC_PI_5);
    p = vec2(p.x.abs(), -p.y);
    p -= 2.0 * K1 * p.dot(K1).min(0.0);
    p -= 2.0 * K2 * p.dot(K2).min(0.0);
    p -= vec2(p.x.clamp(-TAN_FRAC_PI_5 * r, TAN_FRAC_PI_5 * r), r);
    p.length() * p.y.signum()
}

pub fn cross(mut p: Vec2, b: Vec2) -> f32 {
    p = p.abs();
    if p.y > p.x {
        p = p.yx()
    }
    let u = p - b.y;
    let v = p - b;
    if u.x < 0.0 {
        (-u.length()).max(v.x)
    } else if v.x < 0.0 || v.y < 0.0 {
        v.max_element()
    } else {
        v.length()
    }
}

pub fn regular_star(mut p: Vec2, r: f32, n: u32, m: f32) -> f32 {
    let an = PI / (n as f32);
    let en = PI / (m * (n as f32 - 2.0) + 2.0);
    let acs = Vec2::from_angle(an);
    let ecs = Vec2::from_angle(en);

    let (s, c) = (((p.x.atan2(p.y) + PI * 2.0) % (2.0 * an)) - an).sin_cos();
    p = p.length() * vec2(c, s.abs());
    p -= r * acs;
    p += ecs * (-p.dot(ecs)).clamp(0.0, r * acs.y / ecs.y);
    p.length() * p.x.signum()
}

pub fn regular_polygon(mut p: Vec2, r: f32, n: u32) -> f32 {
    let an = PI / (n as f32);
    let acs = Vec2::from_angle(an);

    let (s, c) = (((p.x.atan2(p.y) + PI * 2.0) % (2.0 * an)) - an).sin_cos();
    p = p.length() * vec2(c, s.abs());
    p -= r * acs;
    p.y += (-p.y).clamp(0.0, r * acs.y);
    p.length() * p.x.signum()
}
