use crate::{
    controller::{BindGroupBufferType, BufferData, SSBO},
    egui_components::sdf_builder_tree::{Command, Item, SdfBuilderTree, SdfInstructions},
    window::UserEvent,
};
use bytemuck::Zeroable;
use dfutils::{grid::*, primitives::*, primitives_enum::Shape, sdf::Sdf};
use egui::{Context, CursorIcon};
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
    event_loop::EventLoopProxy,
};
use glam::*;
use shared::{from_pixels, push_constants::sdf_builder::ShaderConstants};
use std::time::Instant;

#[derive(Clone, Copy)]
struct Grabbing {
    position: Vec2,
    derivative: Vec2,
}

impl Grabbing {
    fn new(position: Vec2, derivative: Vec2) -> Self {
        Self {
            position,
            derivative,
        }
    }
}

pub struct Controller {
    size: PhysicalSize<u32>,
    start: Instant,
    shader_constants: ShaderConstants,
    grid: Grid,
    sdf_builder_tree: SdfBuilderTree,
    cursor: Vec2,
    mouse_button_pressed: bool,
    can_grab: bool,
    grabbing: Option<Grabbing>,
    original_selected_item: Option<Item>,
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
            can_grab: false,
            grabbing: None,
            original_selected_item: None,
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
        let cursor = self.cursor_from_pixels();
        if let (
            Some(Grabbing {
                position,
                derivative,
            }),
            Some(item),
        ) = (self.grabbing, &self.original_selected_item)
        {
            match item {
                Item::Shape(shape) => match shape {
                    Shape::Disk(disk) => {
                        let s = (cursor - position) * derivative;
                        if let Some(item_id) = self.sdf_builder_tree.selected_item.0 {
                            self.sdf_builder_tree
                                .command_sender
                                .send(Command::EditItem {
                                    item: Item::Shape(Shape::Disk(Disk::new(
                                        (disk.radius + s.x + s.y).max(0.0),
                                    ))),
                                    item_id,
                                })
                                .ok();
                        }
                    }
                    Shape::Rectangle(rectangle) => {
                        let s = (cursor - position) * derivative;
                        if let Some(item_id) = self.sdf_builder_tree.selected_item.0 {
                            self.sdf_builder_tree
                                .command_sender
                                .send(Command::EditItem {
                                    item: Item::Shape(Shape::Disk(Disk::new(
                                        (rectangle.width + s.x + s.y).max(0.0),
                                    ))),
                                    item_id,
                                })
                                .ok();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_button_pressed = match state {
                ElementState::Pressed => {
                    if self.can_grab {
                        self.grabbing =
                            Some(Grabbing::new(self.cursor_from_pixels(), self.derivative()));
                        self.can_grab = false;
                        self.original_selected_item = self.sdf_builder_tree.selected_item.1.clone();
                    }
                    true
                }
                ElementState::Released => {
                    self.grabbing = None;
                    false
                }
            }
        }
    }

    fn update(&mut self) {
        self.shader_constants = ShaderConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            mouse_button_pressed: (self.mouse_button_pressed && self.grabbing.is_none()).into(),
            cursor: self.cursor_from_pixels().into(),
        }
    }

    fn push_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.shader_constants)
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui(&mut self, ctx: &Context, ui: &mut egui::Ui, event_proxy: &EventLoopProxy<UserEvent>) {
        if self.grabbing.is_some() {
            ctx.set_cursor_icon(CursorIcon::Grabbing);
        } else if let Some(item) = &self.sdf_builder_tree.selected_item.1 {
            match item {
                Item::Shape(shape) => match shape {
                    Shape::Disk(disk) => {
                        if disk.distance(self.cursor_from_pixels()) < 0.01 {
                            ctx.set_cursor_icon(CursorIcon::Grab);
                            self.can_grab = true;
                        } else {
                            self.can_grab = false;
                        }
                    }
                    Shape::Rectangle(rectangle) => {
                        if rectangle.distance(self.cursor_from_pixels()) < 0.01 {
                            ctx.set_cursor_icon(CursorIcon::Grab);
                            self.can_grab = true;
                        } else {
                            self.can_grab = false;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            self.can_grab = false;
            ctx.set_cursor_icon(CursorIcon::Default);
        }

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

impl Controller {
    fn cursor_from_pixels(&self) -> Vec2 {
        from_pixels(self.cursor, self.size.into())
    }

    fn derivative(&self) -> Vec2 {
        if let Some(item) = &self.sdf_builder_tree.selected_item.1 {
            match item {
                Item::Shape(shape) => shape.derivative(self.cursor_from_pixels()),
                _ => Vec2::ZERO,
            }
        } else {
            Vec2::ZERO
        }
    }
}
