use super::{Size, Vec2};
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub size: Size,
    pub cursor: Vec2,
    pub translate: Vec2,
    pub time: f32,
    pub camera_distance: f32,
    /// Bit mask of the pressed buttons (0 = Left, 1 = Middle, 2 = Right).
    pub mouse_button_pressed: u32,
    pub n: u32,
    pub l: u32,
    pub m: i32,
    pub brightness: f32,
    pub normalization_constant: f32,
}
