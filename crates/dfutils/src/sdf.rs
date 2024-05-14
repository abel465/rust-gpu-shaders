use glam::{vec2, Vec2};
#[cfg(not(feature = "std"))]
use num_traits::Float;

pub trait SignedDistance
where
    Self: Sized + Clone + Copy,
{
    fn value(&self) -> f32;

    fn divergent() -> Self;

    fn with_new_distance(&self, d: f32) -> Self;

    fn union(&self, other: &Self) -> Self {
        let a = self.value();
        let b = other.value();
        if a < b {
            *self
        } else {
            *other
        }
    }

    fn intersect(&self, other: &Self) -> Self {
        let a = self.value();
        let b = other.value();
        if a > b {
            *self
        } else {
            *other
        }
    }

    fn subtract(&self, other: &Self) -> Self {
        let a = self.value();
        let b = other.value();
        if -a > b {
            self.with_new_distance(-a)
        } else {
            *other
        }
    }

    fn xor(&self, other: &Self) -> Self {
        self.intersect(other).subtract(&self.union(other))
    }
}

impl SignedDistance for f32 {
    fn value(&self) -> f32 {
        *self
    }

    fn with_new_distance(&self, d: f32) -> Self {
        d
    }

    fn divergent() -> Self {
        f32::INFINITY
    }
}

#[enum_delegate::register]
pub trait Sdf {
    type T: SignedDistance;
    fn signed_distance(&self, p: Vec2) -> Self::T;

    fn distance(&self, p: Vec2) -> Self::T {
        let result = self.signed_distance(p);
        result.with_new_distance(result.value().abs())
    }

    fn derivative(&self, p: Vec2, h: f32) -> Vec2 {
        vec2(
            self.signed_distance(p + h * Vec2::X).value()
                - self.signed_distance(p - h * Vec2::X).value(),
            self.signed_distance(p + h * Vec2::Y).value()
                - self.signed_distance(p - h * Vec2::Y).value(),
        ) / (2.0 * h)
    }
}
