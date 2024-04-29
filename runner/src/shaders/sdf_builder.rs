use crate::{
    controller::{BindGroupBufferType, BufferData, SSBO},
    egui_components::sdf_builder_tree::SdfBuilderTree,
    window::UserEvent,
};
use bytemuck::Zeroable;
use dfutils::grid::*;
use egui::Context;
use egui_winit::winit::{dpi::PhysicalSize, event_loop::EventLoopProxy};
use glam::*;
use shared::push_constants::sdf_builder::ShaderConstants;
use std::time::Instant;

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    shader_constants: ShaderConstants,
    grid: Grid,
    sdf_builder_tree: SdfBuilderTree,
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            start: Instant::now(),
            shader_constants: ShaderConstants::zeroed(),
            grid: Grid::new(size.width as usize, size.height as usize),
            sdf_builder_tree: SdfBuilderTree::default(),
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.sdf_builder_tree.grid_needs_updating = true;
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
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
            self.grid.resize(
                self.size.width as usize,
                self.size.height as usize,
                &self.sdf_builder_tree,
            );
            self.grid.update(&self.sdf_builder_tree);
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
