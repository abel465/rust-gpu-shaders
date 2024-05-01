use crate::{
    gridref::{GridRef, GridRefMut},
    sdf::Sdf,
};
use glam::{vec2, Vec2};

pub struct Grid {
    pub buffer: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

impl Grid {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            buffer: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }

    pub fn as_ref(&self) -> GridRef<'_> {
        GridRef::new(&self.buffer, self.rows, self.cols)
    }

    pub fn as_ref_mut(&mut self) -> GridRefMut<'_> {
        GridRefMut::new(&mut self.buffer, self.rows, self.cols)
    }

    pub fn from_sdf<
        #[cfg(feature = "rayon")] S: Sdf + Sync,
        #[cfg(not(feature = "rayon"))] S: Sdf,
    >(
        rows: usize,
        cols: usize,
        sdf: &S,
    ) -> Self {
        let mut result = Self::new(rows, cols);
        result.update(sdf);
        result
    }

    pub fn update<
        #[cfg(feature = "rayon")] S: Sdf + Sync,
        #[cfg(not(feature = "rayon"))] S: Sdf,
    >(
        &mut self,
        sdf: &S,
    ) {
        #[cfg(feature = "rayon")]
        use rayon::prelude::*;

        let ar = self.aspect_ratio();

        #[cfg(feature = "rayon")]
        let iter = self.buffer.par_iter_mut();
        #[cfg(not(feature = "rayon"))]
        let iter = self.buffer.iter_mut();

        iter.enumerate().for_each(|(i, value)| {
            let row = i / self.cols;
            let col = i - row * self.cols;
            let p = vec2(
                (row as f32 / self.rows as f32 - 0.5) * ar,
                col as f32 / self.cols as f32 - 0.5,
            ) + (self.cols as f32).recip() * 0.5;
            debug_assert!(p.x.abs() < 0.5 * ar && p.y.abs() < 0.5);
            *value = sdf.signed_distance(p);
        });
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;
        let new_size = rows * cols;
        if new_size > self.buffer.len() {
            self.buffer.resize(new_size, 0.0);
        }
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

impl Sdf for Grid {
    fn signed_distance(&self, p: Vec2) -> f32 {
        self.as_ref().signed_distance(p)
    }
}
