use crate::SQRT_3;
use spirv_std::glam::{vec2, Vec2, Vec2Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

pub fn sierpinski_triangle(mut p: Vec2, mut r: f32, m: u32) -> f32 {
    const N: Vec2 = vec2(0.5, -0.5 * SQRT_3);

    let mut d = super::equilateral_triangle(p, r);

    for _ in 0..m {
        p.x = p.x.abs();
        r *= 0.5;
        p += N.yx() * r;
        p -= N * N.dot(p - Vec2::Y * r).min(0.0) * 2.0;
        d = super::equilateral_triangle(p, r);
    }

    d
}
