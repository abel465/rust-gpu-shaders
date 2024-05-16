#![cfg_attr(target_arch = "spirv", no_std)]

use complex::Complex;
use core::f32::consts::FRAC_1_SQRT_2;
use push_constants::hydrogen_wavefunction::ShaderConstants;
use shared::*;
use spherical_harmonics::{factorialu, spherical_harmonic2, to_spherical};
use spirv_std::glam::{vec2, vec3, Mat3, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

// All units of distance are multiples of the Bohr radius

fn laguerre_polynomial(r: u32, s: u32, x: f32) -> f32 {
    let mut sum = 0.0;
    let mut numerator = factorialu(s + r).powi(2);
    let mut denominator = factorialu(s) * factorialu(r);
    for q in 0..=s {
        sum += numerator / denominator;
        numerator *= -x * (s - q) as f32;
        denominator *= (r + q + 1) as f32 * (q + 1) as f32;
    }
    sum
}

fn radial_nc(n: u32, l: u32) -> f32 {
    ((2.0 / n as f32).powi(3) * factorialu(n - l - 1)
        / (2.0 * n as f32 * factorialu(n + l).powi(3)))
    .sqrt()
}

fn angular_nc(m: i32, l: u32) -> f32 {
    spherical_harmonics::normalization_constant(m, l)
}

fn radial_wavefunction(n: u32, l: u32, r: f32) -> f32 {
    let p = 2.0 * r / n as f32;
    let asymptotic_forms = (-r / n as f32).exp() * p.powi(l as i32);
    let lp = laguerre_polynomial(2 * l + 1, n - l - 1, p);
    asymptotic_forms * lp
}

fn hydrogen_wavefunction(n: u32, l: u32, m: i32, r: f32, theta: f32, phi: f32) -> Complex {
    let radial = radial_wavefunction(n, l, r);
    let angular = spherical_harmonic2(m, l, theta, phi);
    radial * angular
}

// this function is intentionally overstating the integral for visualization purposes
pub fn integrate_ray(n: u32, l: u32, m: i32, ro: Vec3, rd: Vec3, camera_distance: f32) -> Complex {
    let num_samples = 100;
    let delta_z = 2.0 * camera_distance / num_samples as f32;
    let mut integral = Complex::ZERO;
    let mut pos = ro;
    for _ in 0..num_samples {
        let (r, theta, phi) = to_spherical(pos);
        integral += hydrogen_wavefunction(n, l, m, r, theta, phi);
        pos += rd * delta_z;
    }
    radial_nc(n, l) * angular_nc(m, l) * integral * delta_z
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let translate: Vec2 = constants.translate.into();
    let uv = from_pixels(frag_coord.xy(), constants.size);

    let rm = Mat3::from_rotation_y(translate.x).mul_mat3(&Mat3::from_rotation_x(translate.y));
    let ro = rm * Vec2::ZERO.extend(-constants.camera_distance);
    let rd = rm * uv.extend(1.0).normalize();

    let z = integrate_ray(
        constants.n,
        constants.l,
        constants.m,
        ro,
        rd,
        constants.camera_distance,
    ) * Complex::from_angle(constants.time);

    let col = vec3(
        z.dot(Vec2::X),
        z.dot(vec2(-FRAC_1_SQRT_2, FRAC_1_SQRT_2)),
        z.dot(Vec2::splat(-FRAC_1_SQRT_2)),
    )
    .powf(1.0 / constants.brightness as f32);

    *output = col.extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    fullscreen_vs(vert_id, out_pos)
}

#[cfg(test)]
mod test {
    use super::*;
    use numeric_integration::integrate_3d;
    use shared::assert_similar;

    #[test]
    fn test_hydrogen_wavefunction() {
        for n in 0..5 {
            for l in 0u32..n {
                for m in 1 - (l as i32)..=l as i32 {
                    let f = |pos: Vec3| {
                        let (r, theta, phi) = to_spherical(pos);
                        let nc = radial_nc(n, l) * angular_nc(m, l);
                        let v = nc * hydrogen_wavefunction(n, l, m, r, theta, phi);
                        v.norm_squared()
                    };
                    let d = 35.0;
                    let total_probability =
                        integrate_3d(&f, Vec3::splat(-d), Vec3::splat(d), [25; 3]);
                    // Accuracy increases with more samples
                    assert_similar!(total_probability, 1.0, 0.02);
                }
            }
        }
    }
}
