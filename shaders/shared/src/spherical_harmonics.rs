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

fn general_binomial(n: f32, k: u32) -> f32 {
    let mut x = 1.0;
    for i in 0..k {
        x *= (n - i as f32) / (i + 1) as f32;
    }
    x
}

fn legendre_polynomial(m: i32, l: u32, x: f32) -> Complex {
    fn legendre_polynomial_positive(m: u32, l: u32, x: f32) -> Complex {
        let mut sm = 0.0;
        let mut denominator = 1.0;
        let mut numerator = {
            let mut x = 1.0;
            for i in 1..=m {
                x *= (l + 1 - i) as f32;
            }
            x
        };
        for k in m..=l {
            sm += numerator / denominator * general_binomial(((l + k) as f32 - 1.0) / 2.0, l);
            numerator *= x * (k + 1) as f32 * (l - k) as f32;
            denominator *= (k - m + 1) as f32 * (k + 1) as f32;
        }
        let z = if m == 0 {
            Complex::ONE
        } else {
            let exp = m as f32 / 2.0;
            let x = 1.0 - x * x;
            let r = x.abs().powf(exp);
            let theta = exp * 0.0_f32.atan2(x);
            r * Complex::from_angle(theta)
        };

        (-1.0_f32).powi(m as i32) * 2.0_f32.powi(l as i32) * z * sm
    }
    if m < 0 {
        (-1.0_f32).powi(-m) * factorialu((l as i32 + m) as u32) / factorialu((l as i32 - m) as u32)
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
    let normalization_constant = (((2 * l + 1) as f32 * factorialu((l as i32 - m) as u32))
        / (4.0 * PI * factorialu((l as i32 + m) as u32)))
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

pub fn normalization_constant(m: i32, l: u32) -> f32 {
    (((2 * l + 1) as f32 * factorialu((l as i32 - m) as u32))
        / (4.0 * PI * factorialu((l as i32 + m) as u32)))
    .sqrt()
}

/// like `spherical_harmonic` but unnormalized
pub fn spherical_harmonic2(m: i32, l: u32, theta: f32, phi: f32) -> Complex {
    let angular = Complex::from_angle(phi * m as f32);
    let lp = legendre_polynomial(m, l, theta.cos());
    lp * angular
}

/// like `real_spherical_harmonic` but accepts a precomputed normalization constant
pub fn real_spherical_harmonic2(m: i32, l: u32, theta: f32, phi: f32, nc: Complex) -> f32 {
    let sh = nc * spherical_harmonic2(m.abs(), l, theta, phi);
    if m == 0 {
        sh.x
    } else if m > 0 {
        2.0_f32.sqrt() * sh.x
    } else {
        2.0_f32.sqrt() * sh.y
    }
}
