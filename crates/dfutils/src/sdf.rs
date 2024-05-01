use glam::{vec2, Vec2};
#[cfg(not(feature = "std"))]
use num_traits::Float;

pub trait Sdf {
    fn signed_distance(&self, p: Vec2) -> f32;

    fn distance(&self, p: Vec2) -> f32 {
        self.signed_distance(p).abs()
    }

    fn derivative(&self, p: Vec2) -> Vec2 {
        const H: f32 = 0.001;
        vec2(
            self.signed_distance(p + H * Vec2::X) - self.signed_distance(p - H * Vec2::X),
            self.signed_distance(p + H * Vec2::Y) - self.signed_distance(p - H * Vec2::Y),
        ) / (2.0 * H)
    }
}
