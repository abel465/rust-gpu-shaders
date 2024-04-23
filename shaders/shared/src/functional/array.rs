pub use super::traits::*;
use crate::reduce;

/// Arrays have these methods but dont work with rust-gpu

pub trait ArrayMap<T, U> {
    const N: usize;
    fn map2<F>(self, f: F) -> [U; Self::N]
    where
        F: Fn(T) -> U,
        T: Copy;
}

pub trait ArrayZip<T>
where
    T: Copy,
{
    type Output;
    fn zip2(self, other: Self) -> Self::Output;
}

macro_rules! impl_array {(
    $N:literal, $($idx:literal)*
) => {
        impl<T, U> ArrayMap<T, U> for [T; $N] {
            const N: usize = $N;
            fn map2<F>(self, f: F) -> [U; $N]
            where
                F: Fn(T) -> U, T: Copy,
            {
                [$(f(self[$idx])),*]
            }
        }
        impl<T> ArrayZip<T> for [T; $N] where T: Copy {
            type Output = [[T; 2]; $N];
            fn zip2(self, other: Self) -> Self::Output
             {
                 [$([self[$idx], other[$idx]]),*]
            }
        }
        impl MinElement for [f32; $N]
        {
            type Output = f32;
            fn min_element(self) -> Self::Output {
                reduce!((f32::min),$(self[$idx]),*)
            }
        }
        impl MaxElement for [f32; $N]
        {
            type Output = f32;
            fn max_element(self) -> Self::Output {
                reduce!((f32::max),$(self[$idx]),*)
            }
        }
    }
}

impl_array! { 2, 0 1 }
impl_array! { 3, 0 1 2 }
impl_array! { 4, 0 1 2 3 }
impl_array! { 5, 0 1 2 3 4 }
impl_array! { 6, 0 1 2 3 4 5 }
