pub use super::traits::*;
use crate::{reduce, saturate};
use core::ops::*;
use spirv_std::glam::{Vec2, Vec3};
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

pub trait Projection {
    fn project_onto_segment(self, rhs: Self) -> Self;
    fn reject_from_segment(self, rhs: Self) -> Self;
    fn reflect(self, n: Self) -> Self;
    fn refract(self, n: Self, eta: f32) -> Self;
}

macro_rules! impl_vec {
    ($($T:ty)+) => {$(
        impl Projection for $T {
            fn project_onto_segment(self, rhs: Self) -> Self {
                rhs * saturate(self.dot(rhs) / rhs.length_squared())
            }

            fn reject_from_segment(self, rhs: Self) -> Self {
                self - self.project_onto_segment(rhs)
            }

            fn reflect(self, n: Self) -> Self {
                self - self.project_onto_normalized(n) * 2.0
            }

            fn refract(self, n: Self, eta: f32) -> Self {
                let k = 1.0 - eta * eta * (1.0 - self.dot(n) * self.dot(n));
                if k < 0.0 {
                    Self::ZERO
                } else {
                    eta * self - (eta * self.dot(n) + k.sqrt()) * n
                }
            }
        }
    )+}
}

impl_vec!(Vec2 Vec3);

macro_rules! impl_vec_with_dimensions {
    ($T:ty, $($d:tt)+) => {
        impl Map<f32, f32> for $T {
            type Output = Self;
            fn map<F>(self, f: F) -> Self::Output
            where
                F: Fn(f32) -> f32,
            {
                <$T>::new($(f(self.$d)),+)
            }
        }
        impl Sum for $T {
            type Output = f32;
            fn sum(self) -> Self::Output {
                reduce!((f32::add), $(self.$d),+)
            }
        }
        impl Product for $T {
            type Output = f32;
            fn product(self) -> Self::Output {
                reduce!((f32::mul), $(self.$d),+)
            }
        }
    }
}

impl_vec_with_dimensions!(Vec2, x y);
impl_vec_with_dimensions!(Vec3, x y z);
