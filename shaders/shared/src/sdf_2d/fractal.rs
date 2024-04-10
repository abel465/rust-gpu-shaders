use crate::SQRT_3;
use core::f32::consts::PI;
use spirv_std::glam::{vec2, vec3, Mat2, Vec2};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

// Based on https://www.shadertoy.com/view/fdBcR3 (Jakob Thomsen 2/2/2022)
pub fn sierpinski_triangle(p: Vec2, r: f32, m: u32) -> f32 {
    let q = {
        let p = p / r / SQRT_3;
        let z = p.y / SQRT_3;
        vec3(-z - p.x, -z + p.x, 2.0 * z) + 1.0 / 3.0
    };
    if q.x < 0.0 || q.y < 0.0 || q.z < 0.0 {
        super::equilateral_triangle(p, r)
    } else {
        let n = (m as f32).exp2();
        let i = ((1.0 - q) * n).as_ivec3();
        let f = i.x & i.y & i.z;
        if f == 0 {
            -(q % (1.0 / n)).min_element() * r * 3.0 / 2.0
        } else {
            let s = (f as f32).log2().floor().exp2() * 2.0 / n;
            (((1.0 - q) * 2.0) % s).min_element() * r * 3.0 / 4.0
        }
    }
}

// Based on https://www.shadertoy.com/view/NljfRG (sylvain69780 12/05/2022)
pub fn koch_snowflake(mut p: Vec2, r: f32, n: u32) -> f32 {
    p = Mat2::from_angle(PI / 3.0) * p.abs() - r * vec2(0.0, 0.5);
    let rot = Mat2::from_angle(-PI / 6.0);
    let mut w = rot.x_axis.x * r;
    for _ in 0..n * 2 + 2 {
        p = rot * vec2(p.x.abs() - w, -p.y);
        w /= SQRT_3;
        p.x += w;
    }
    p.y.signum() * (vec2(p.x - p.x.clamp(-w, w), p.y)).length()
}
