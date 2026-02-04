use crate::controller::{BindGroupBufferType, BufferData, SSBO};
use bytemuck::Zeroable;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
};
use glam::{vec2, Vec2};
use shared::{
    interpreter::{OpCode0, OpCodeStruct},
    push_constants::fun_rep_demo::ShaderConstants,
};
use std::time::{Duration, Instant};

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    elapsed: Duration,
    cursor: Vec2,
    mouse_button_pressed: bool,
    can_drag: Option<usize>,
    drag_point: Option<usize>,
    shader_constants: ShaderConstants,
    zoom: f32,
    buffer: Vec<OpCodeStruct>,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        let ops: Vec<OpCodeStruct> = sdf().iter().map(|op| (*op).into()).collect();

        Self {
            size,
            start: Instant::now(),
            elapsed: Duration::ZERO,
            cursor: Vec2::ZERO,
            mouse_button_pressed: false,
            can_drag: None,
            drag_point: None,
            shader_constants: ShaderConstants::zeroed(),
            zoom: 1.0,
            buffer: ops,
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    self.drag_point = self.can_drag;
                    self.mouse_button_pressed = true;
                }
                ElementState::Released => {
                    self.drag_point = None;
                    self.mouse_button_pressed = false;
                }
            }
        }
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
    }

    fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        self.zoom *= match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let v = 1.0 + 0.1 * y.abs();
                if y < 0.0 {
                    v
                } else {
                    1.0 / v
                }
            }
            MouseScrollDelta::PixelDelta(p) => {
                let v = 1.0 + 0.02 * (1.0 + p.y.abs() as f32).ln();
                if p.y < 0.0 {
                    v
                } else {
                    1.0 / v
                }
            }
        };
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size
    }

    fn update(&mut self) {
        self.elapsed = self.start.elapsed();
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.elapsed.as_secs_f32(),
            cursor: self.cursor.into(),
            mouse_button_pressed: !(1
                << (self.mouse_button_pressed && self.drag_point.is_none()) as u32),
            zoom: self.zoom,
        };
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        false
    }

    fn buffers(&self) -> BufferData<'_> {
        BufferData {
            bind_group_buffers: vec![BindGroupBufferType::SSBO(SSBO {
                data: bytemuck::cast_slice(&self.buffer),
                read_only: true,
            })],
            ..Default::default()
        }
    }
}

fn length() -> Vec<OpCode0> {
    use OpCode0::*;
    vec![Pushx, Square, Pushy, Square, Add, Sqrt]
}

fn disk(r: f32) -> Vec<OpCode0> {
    use OpCode0::*;
    let mut vec = length();
    vec.extend(vec![Push(r), Sub]);
    vec
}

fn sdf() -> Vec<OpCode0> {
    disk(0.3)
}
