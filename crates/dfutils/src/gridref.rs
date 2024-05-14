use glam::Vec2;
#[cfg(not(feature = "std"))]
use num_traits::Float;

#[derive(Clone, Copy)]
pub struct GridRef<'a, T> {
    w: usize,
    h: usize,
    buffer: &'a [T],
}

impl<'a, T: Copy> GridRef<'a, T> {
    pub fn new(w: usize, h: usize, buffer: &'a [T]) -> Self {
        Self { w, h, buffer }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h as f32
    }

    pub fn get(&self, x: usize, y: usize) -> T {
        self.buffer[y * self.w + x]
    }

    pub fn signed_distance(&self, p: Vec2) -> T {
        let ar = self.aspect_ratio();
        debug_assert!(p.x.abs() < 0.5 * ar && p.y.abs() < 0.5);
        let x = ((p.x + 0.5 * ar) / ar * self.w as f32) as usize;
        let y = ((0.5 - p.y) * self.h as f32) as usize;
        self.get(x, y)
    }
}

pub struct GridRefMut<'a, T> {
    w: usize,
    h: usize,
    buffer: &'a mut [T],
}

impl<'a, T: Copy> GridRefMut<'a, T> {
    pub fn new(w: usize, h: usize, buffer: &'a mut [T]) -> Self {
        Self { w, h, buffer }
    }

    pub fn as_ref(&self) -> GridRef<'_, T> {
        GridRef::new(self.w, self.h, self.buffer)
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h as f32
    }

    pub fn get(&self, x: usize, y: usize) -> T {
        self.buffer[y * self.w + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.buffer[y * self.w + x] = value;
    }

    pub fn signed_distance(&self, p: Vec2) -> T {
        self.as_ref().signed_distance(p)
    }
}
