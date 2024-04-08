use crate::complex::Complex;
use core::f32::consts::PI;
use spirv_std::glam::{vec3, Vec3, Vec3Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

pub fn factorialu(n: u32) -> f32 {
    let mut x = 1.0;
    for i in 2..=n {
        x *= i as f32;
    }
    x
}

fn binomial(n: u32, k: u32) -> f32 {
    let mut x = 1.0;
    for i in 1..=k {
        x *= (n + 1 - i) as f32 / i as f32;
    }
    x
}

fn general_binomial(n: f32, k: u32) -> f32 {
    let mut x = 1.0;
    for i in 0..k {
        x *= n - i as f32;
    }
    x / factorialu(k)
}

fn legendre_polynomial(m: i32, l: u32, x: f32) -> Complex {
    fn legendre_polynomial_positive(m: u32, l: u32, x: f32) -> Complex {
        let mut sm = 0.0;
        for k in m..=l {
            sm += factorialu(k) / factorialu(k - m)
                * x.powi((k - m) as i32)
                * binomial(l, k)
                * general_binomial(((l + k) as f32 - 1.0) / 2.0, l);
        }
        let bb = Complex::new(1.0 - x * x, 0.0).powf(m as f32 / 2.0);
        (-1.0_f32).powi(m as i32) * 2.0_f32.powi(l as i32) * bb * sm
    }
    if m < 0 {
        (-1.0_f32).powi(-m) * factorialu(l + m as u32) / factorialu(l - m as u32)
            * legendre_polynomial_positive((-m) as u32, l, x)
    } else {
        legendre_polynomial_positive(m as u32, l, x)
    }
}

pub fn from_spherical(r: f32, theta: f32, phi: f32) -> Vec3 {
    let (st, ct) = theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    r * vec3(sp * ct, sp * st, cp)
}

pub fn to_spherical(pos: Vec3) -> (f32, f32, f32) {
    let r = pos.length();
    let theta = pos.xy().length().atan2(pos.z);
    let phi = pos.y.atan2(pos.x);
    (r, theta, phi)
}

pub fn spherical_harmonic(m: i32, l: u32, theta: f32, phi: f32, time: f32) -> Complex {
    let normalization_constant = (((2 * l + 1) as f32 * factorialu(l - m as u32))
        / (4.0 * PI * factorialu(l + m as u32)))
    .sqrt();
    let angular = Complex::from_angle(phi * m as f32);
    let lp = legendre_polynomial(m, l, theta.cos());
    normalization_constant * lp * angular * Complex::from_angle(time)
}

pub fn real_spherical_harmonic(m: i32, l: u32, theta: f32, phi: f32, time: f32) -> f32 {
    let sh = spherical_harmonic(m.abs(), l, theta, phi, time);
    if m == 0 {
        sh.x
    } else if m > 0 {
        2.0_f32.sqrt() * sh.x
    } else {
        2.0_f32.sqrt() * sh.y
    }
}
