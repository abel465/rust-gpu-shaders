use super::{Bool, Size, Vec2};
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub size: Size,
    pub cursor: Vec2,
    pub time: f32,
    pub mouse_button_pressed: Bool,
    pub selected_id: u32,
}
