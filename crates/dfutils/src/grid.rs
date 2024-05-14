use crate::{
    gridref::{GridRef, GridRefMut},
    sdf::Sdf,
};
use glam::{vec2, Vec2};

pub struct Grid<T> {
    pub w: usize,
    pub h: usize,
    pub buffer: Vec<T>,
}

impl<#[cfg(feature = "rayon")] T: Send, #[cfg(not(feature = "rayon"))] T> Grid<T>
where
    T: Default + Clone + Copy,
{
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            buffer: vec![Default::default(); w * h],
        }
    }

    pub fn as_ref(&self) -> GridRef<'_, T> {
        GridRef::new(self.w, self.h, &self.buffer)
    }

    pub fn as_ref_mut(&mut self) -> GridRefMut<'_, T> {
        GridRefMut::new(self.w, self.h, &mut self.buffer)
    }

    pub fn from_sdf<
        #[cfg(feature = "rayon")] S: Sdf<T = T> + Sync,
        #[cfg(not(feature = "rayon"))] S: Sdf<T = T>,
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
        #[cfg(feature = "rayon")] S: Sdf<T = T> + Sync,
        #[cfg(not(feature = "rayon"))] S: Sdf<T = T>,
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
            let y = i / self.w;
            let x = i - y * self.w;
            let p = vec2(
                (x as f32 / self.w as f32 - 0.5) * ar,
                0.5 - y as f32 / self.h as f32,
            ) + 0.5 / self.w as f32;
            debug_assert!(p.x.abs() < 0.5 * ar && p.y.abs() < 0.5);
            *value = sdf.signed_distance(p);
        });
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        let new_size = w * h;
        if new_size > self.buffer.len() {
            self.buffer.resize(new_size, Default::default());
        }
    }

    fn aspect_ratio(&self) -> f32 {
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
