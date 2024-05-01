pub use crate::sdf::Sdf;
use glam::Vec2;
#[cfg(feature = "libm")]
use num_traits::Float;

pub struct GridRef<'a> {
    buffer: &'a [f32],
    rows: usize,
    cols: usize,
}

impl<'a> GridRef<'a> {
    pub fn new(buffer: &'a [f32], rows: usize, cols: usize) -> Self {
        Self { buffer, rows, cols }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.rows as f32 / self.cols as f32
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.buffer[row * self.cols + col]
    }
}

impl<'a> Sdf for GridRef<'a> {
    fn signed_distance(&self, p: Vec2) -> f32 {
        let ar = self.aspect_ratio();
        debug_assert!(p.x.abs() < 0.5 * ar && p.y.abs() < 0.5);
        let row = ((p.x + 0.5 * ar) / ar * self.rows as f32) as usize;
        let col = ((p.y + 0.5) * self.cols as f32) as usize;
        self.get(row, col)
    }
}

pub struct GridRefMut<'a> {
    buffer: &'a mut [f32],
    rows: usize,
    cols: usize,
}

impl<'a> GridRefMut<'a> {
    pub fn new(buffer: &'a mut [f32], rows: usize, cols: usize) -> Self {
        Self { buffer, rows, cols }
    }

    pub fn as_ref(&self) -> GridRef<'_> {
        GridRef::new(&self.buffer, self.rows, self.cols)
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.rows as f32 / self.cols as f32
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.buffer[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.buffer[row * self.cols + col] = value;
    }
}

impl<'a> Sdf for GridRefMut<'a> {
    fn signed_distance(&self, p: Vec2) -> f32 {
        self.as_ref().signed_distance(p)
    }
}
