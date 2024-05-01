use glam::Vec2;
#[cfg(not(feature = "std"))]
use num_traits::Float;

pub trait Sdf {
    fn signed_distance(&self, p: Vec2) -> f32;
    fn distance(&self, p: Vec2) -> f32 {
        self.signed_distance(p).abs()
    }
}
