use crate::sdf::Sdf;
use glam::*;
#[cfg(not(feature = "std"))]
use num_traits::Float;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cross {
    pub length: f32,
    pub thickness: f32,
}

impl Cross {
    pub const fn new(length: f32, thickness: f32) -> Self {
        Self { length, thickness }
    }
}

impl Default for Cross {
    fn default() -> Self {
        Self {
            length: 0.3,
            thickness: 0.1,
        }
    }
}

impl Sdf for Cross {
    fn signed_distance(&self, mut p: Vec2) -> f32 {
        p = p.abs();
        if p.y > p.x {
            p = p.yx()
        }
        let u = p - self.thickness;
        let v = p - vec2(self.length, self.thickness);
        if u.x < 0.0 {
            (-u.length()).max(v.x)
        } else if v.x < 0.0 || v.y < 0.0 {
            v.max_element()
        } else {
            v.length()
        }
    }

    fn distance(&self, mut p: Vec2) -> f32 {
        p = p.abs();
        if p.y > p.x {
            p = p.yx()
        }
        let u = p - self.thickness;
        let v = p - vec2(self.length, self.thickness);
        if u.x < 0.0 {
            u.length().min(-v.x)
        } else if v.x < 0.0 || v.y < 0.0 {
            v.max_element().abs()
        } else {
            v.length()
        }
    }
}
