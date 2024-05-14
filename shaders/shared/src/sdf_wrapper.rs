use dfutils::sdf::*;
use spirv_std::glam::*;

#[derive(Clone, Copy)]
pub struct SdfWrapper<S, T>
where
    S: Sdf<T = f32>,
{
    sdf: S,
    data: T,
}

impl<S, T> SdfWrapper<S, T>
where
    S: Sdf<T = f32>,
{
    pub fn new(sdf: S, data: T) -> Self {
        Self { sdf, data }
    }
}

impl<S, T> Sdf for SdfWrapper<S, T>
where
    S: Sdf<T = f32>,
    T: Copy + Default,
{
    type T = WrappedDistance<T>;
    fn signed_distance(&self, p: Vec2) -> WrappedDistance<T> {
        WrappedDistance::new(self.sdf.signed_distance(p), self.data)
    }

    fn distance(&self, p: Vec2) -> WrappedDistance<T> {
        WrappedDistance::new(self.sdf.distance(p), self.data)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WrappedDistance<T> {
    pub d: f32,
    pub data: T,
}

impl<T> WrappedDistance<T> {
    pub fn new(d: f32, data: T) -> Self {
        Self { d, data }
    }
}

impl<T: Copy + Default> SignedDistance for WrappedDistance<T> {
    fn value(&self) -> f32 {
        self.d
    }

    fn with_new_distance(&self, d: f32) -> Self {
        Self::new(d, self.data)
    }

    fn divergent() -> Self {
        Self::new(f32::INFINITY, Default::default())
    }
}
