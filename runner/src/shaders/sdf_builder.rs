use crate::{
    controller::{BindGroupBufferType, BufferData, SSBO},
    egui_components::sdf_builder_tree::{SdfBuilderTree, SdfInstructions},
    window::UserEvent,
};
use bytemuck::Zeroable;
use dfutils::grid::*;
use egui::Context;
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
    event_loop::EventLoopProxy,
};
use glam::*;
use shared::{from_pixels, push_constants::sdf_builder::ShaderConstants};
use std::time::Instant;

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    shader_constants: ShaderConstants,
    grid: Grid,
    sdf_builder_tree: SdfBuilderTree,
    cursor: Vec2,
    mouse_button_pressed: bool,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            shader_constants: ShaderConstants::zeroed(),
            grid: Grid::new(size.width as usize, size.height as usize),
            sdf_builder_tree: SdfBuilderTree::default(),
            cursor: Vec2::ZERO,
            mouse_button_pressed: false,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.grid
            .resize(self.size.width as usize, self.size.height as usize);
        self.sdf_builder_tree.grid_needs_updating = true;
    }

    fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32);
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_button_pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            mouse_button_pressed: self.mouse_button_pressed.into(),
            cursor: from_pixels(self.cursor, self.size.into()).into(),
        }
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut egui::Ui, event_proxy: &EventLoopProxy<UserEvent>) {
        self.sdf_builder_tree.ui(ui);
        if self.sdf_builder_tree.grid_needs_updating {
            let sdf_instructions =
                SdfInstructions::new(self.sdf_builder_tree.generate_instructions());
            self.grid.update(&sdf_instructions);
            if event_proxy.send_event(UserEvent::NewBuffersReady).is_err() {
                panic!("Event loop dead");
            }
            self.sdf_builder_tree.grid_needs_updating = false;
        }
    }

    fn buffers(&self) -> BufferData {
        BufferData {
            bind_group_buffers: vec![BindGroupBufferType::SSBO(SSBO {
                data: bytemuck::cast_slice(&self.grid.buffer[..]),
                read_only: true,
            })],
            ..Default::default()
        }
    }
}
