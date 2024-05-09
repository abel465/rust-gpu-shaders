pub use crate::sdf::Sdf;
use glam::Vec2;
#[cfg(not(feature = "std"))]
use num_traits::Float;

#[derive(Clone, Copy)]
pub struct GridRef<'a> {
    w: usize,
    h: usize,
    buffer: &'a [f32],
}

impl<'a> GridRef<'a> {
    pub fn new(w: usize, h: usize, buffer: &'a [f32]) -> Self {
        Self { w, h, buffer }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h as f32
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.buffer[y * self.w + x]
    }
}

impl<'a> Sdf for GridRef<'a> {
    fn signed_distance(&self, p: Vec2) -> f32 {
        let ar = self.aspect_ratio();
        debug_assert!(p.x.abs() < 0.5 * ar && p.y.abs() < 0.5);
        let x = ((p.x + 0.5 * ar) / ar * self.w as f32) as usize;
        let y = ((0.5 - p.y) * self.h as f32) as usize;
        self.get(x, y)
    }
}

pub struct GridRefMut<'a> {
    w: usize,
    h: usize,
    buffer: &'a mut [f32],
}

impl<'a> GridRefMut<'a> {
    pub fn new(w: usize, h: usize, buffer: &'a mut [f32]) -> Self {
        Self { w, h, buffer }
    }

    pub fn as_ref(&self) -> GridRef<'_> {
        GridRef::new(self.w, self.h, self.buffer)
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h as f32
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.buffer[y * self.w + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        self.buffer[y * self.w + x] = value;
    }
}

impl<'a> Sdf for GridRefMut<'a> {
    fn signed_distance(&self, p: Vec2) -> f32 {
        self.as_ref().signed_distance(p)
    }
}
