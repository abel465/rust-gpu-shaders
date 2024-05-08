use crate::{
    controller::{BindGroupBufferType, BufferData, SSBO},
    window::UserEvent,
};
use bytemuck::Zeroable;
use dfutils::{grid::*, primitives_enum::Shape, sdf::Sdf};
use egui::{Context, CursorIcon};
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
    event_loop::EventLoopProxy,
};
use glam::*;
use icons::TextureHandles;
use instructions::{id_of_signed_distance, InstructionForId};
use resize::Resize;
use sdf_builder_tree::{Command, Item, ItemId, SdfBuilderTree, SelectedItem};
use shared::{
    from_pixels,
    push_constants::sdf_builder::ShaderConstants,
    sdf_interpreter::{SdfInstructions, Transform},
};
use std::time::{Duration, Instant};

mod icons;
mod instructions;
mod resize;
mod sdf_builder_tree;
pub mod shape_ui;

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

#[derive(PartialEq)]
enum GrabType {
    Move,
    Resize,
    None,
}

impl GrabType {
    fn can_grab(&self) -> bool {
        *self != Self::None
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
    grab_type: GrabType,
    grabbing: Option<Grabbing>,
    original_selected_item: Option<Item>,
    texture_handles: TextureHandles,
    instructions_for_id: Vec<InstructionForId>,
    last_mouse_press: (Vec2, std::time::Instant),
}

impl crate::controller::Controller for Controller {
    fn new(size: PhysicalSize<u32>) -> Self {
        let now = Instant::now();
        Self {
            size,
            start: now,
            shader_constants: ShaderConstants::zeroed(),
            grid: Grid::new(size.width as usize, size.height as usize),
            sdf_builder_tree: SdfBuilderTree::default(),
            cursor: Vec2::ZERO,
            mouse_button_pressed: false,
            grab_type: GrabType::None,
            grabbing: None,
            original_selected_item: None,
            texture_handles: TextureHandles::empty(),
            instructions_for_id: vec![],
            last_mouse_press: (Vec2::ZERO, now),
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
            Some(item_id),
        ) = (
            self.grabbing,
            &self.original_selected_item,
            self.sdf_builder_tree.selected_item.id,
        ) {
            let item: Item = match self.grab_type {
                GrabType::Move => match item {
                    Item::Shape(shape, transform) => Item::Shape(
                        *shape,
                        Transform {
                            position: transform.position - (position - cursor),
                        },
                    ),
                    Item::Operator(_, _) => todo!(),
                },
                GrabType::Resize => match item {
                    Item::Shape(shape, transform) => Item::Shape(
                        shape
                            .resize(
                                position - transform.position,
                                cursor - transform.position,
                                derivative,
                            )
                            .into(),
                        *transform,
                    ),
                    _ => todo!(),
                },
                GrabType::None => unimplemented!(),
            };
            self.sdf_builder_tree
                .send_command(Command::EditItem { item, item_id });
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_button_pressed = match state {
                ElementState::Pressed => {
                    if self.grab_type.can_grab() {
                        self.grabbing = Some(Grabbing::new(
                            self.cursor_from_pixels(),
                            self.derivative_at_cursor(),
                        ));
                        self.original_selected_item = self
                            .sdf_builder_tree
                            .get_selected_item()
                            .map(|item| item.clone());
                    }
                    self.last_mouse_press = (self.cursor, Instant::now());
                    true
                }
                ElementState::Released => {
                    self.grab_type = GrabType::None;
                    self.grabbing = None;
                    let (press_position, instant) = self.last_mouse_press;
                    if press_position.distance_squared(self.cursor) < 4.0
                        && instant.elapsed() < Duration::from_millis(300)
                    {
                        self.sdf_builder_tree.send_command(Command::SetSelectedItem(
                            if let Some(item_id) = self.get_item_id_under_cursor() {
                                item_id.into()
                            } else {
                                SelectedItem::NONE
                            },
                        ));
                    }
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
        self.init_icon_textures(ctx);
        if self.grabbing.is_some() {
            match self.grab_type {
                GrabType::Move => {
                    ctx.set_cursor_icon(CursorIcon::Grabbing);
                }
                GrabType::Resize => {
                    ctx.set_cursor_icon(self.choose_resize_cursor());
                }
                GrabType::None => {}
            }
        } else if let Some(item) = &self.sdf_builder_tree.get_selected_item() {
            match item {
                Item::Shape(shape, transform) => {
                    self.set_grab_type(ctx, *shape, self.cursor_from_pixels() - transform.position);
                }
                _ => {}
            }
        } else {
            self.grab_type = GrabType::None;
            ctx.set_cursor_icon(CursorIcon::Default);
        }

        self.sdf_builder_tree
            .ui(ui, &self.texture_handles, self.size);
        if self.sdf_builder_tree.grid_needs_updating {
            let (instructions, instructions_for_id) = self.sdf_builder_tree.generate_instructions();
            self.instructions_for_id = instructions_for_id;
            self.grid.update(&SdfInstructions::new(&instructions));
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

    fn derivative_at_cursor(&self) -> Vec2 {
        if let Some(item) = &self.sdf_builder_tree.get_selected_item() {
            match item {
                Item::Shape(shape, transform) => {
                    shape.derivative(self.cursor_from_pixels() - transform.position, 0.01)
                }
                _ => Vec2::ZERO,
            }
        } else {
            Vec2::ZERO
        }
    }

    fn choose_resize_cursor(&self) -> CursorIcon {
        const H: f32 = 4.0;
        let d = self.grabbing.map_or(
            self.derivative_at_cursor(),
            |Grabbing { derivative, .. }| derivative,
        );
        let slope = d.y / d.x;
        if slope > 1.0 / H && slope < H {
            CursorIcon::ResizeNeSw
        } else if slope < -1.0 / H && slope > -H {
            CursorIcon::ResizeNwSe
        } else if slope.abs() > 1.0 {
            CursorIcon::ResizeVertical
        } else {
            CursorIcon::ResizeHorizontal
        }
    }

    fn init_icon_textures(&mut self, ctx: &Context) {
        if self.texture_handles.is_empty() {
            let icon_images = icons::generate_icons();
            self.texture_handles = TextureHandles {
                shapes: icon_images
                    .shapes
                    .into_iter()
                    .map(|icon| ctx.load_texture("logo", icon, Default::default()))
                    .collect(),
                operators: icon_images
                    .operators
                    .into_iter()
                    .map(|icon| ctx.load_texture("logo", icon, Default::default()))
                    .collect(),
            }
        }
    }

    fn set_grab_type(&mut self, ctx: &Context, shape: Shape, position: Vec2) {
        let d = shape.signed_distance(position);
        self.grab_type = match shape {
            Shape::LineSegment(line_segment) => {
                if d.abs() < 0.01 {
                    if line_segment.a.distance(position) < 0.01
                        || line_segment.b.distance(position) < 0.01
                    {
                        ctx.set_cursor_icon(self.choose_resize_cursor());
                        GrabType::Resize
                    } else {
                        ctx.set_cursor_icon(CursorIcon::Grab);
                        GrabType::Move
                    }
                } else {
                    GrabType::None
                }
            }
            Shape::Plane(_) => {
                if d < 0.001 {
                    ctx.set_cursor_icon(CursorIcon::Grab);
                    GrabType::Move
                } else {
                    GrabType::None
                }
            }
            Shape::Ray(_) => {
                if position.length() < 0.01 {
                    ctx.set_cursor_icon(CursorIcon::Grab);
                    GrabType::Move
                } else if d.abs() < 0.01 {
                    ctx.set_cursor_icon(self.choose_resize_cursor());
                    GrabType::Resize
                } else {
                    GrabType::None
                }
            }
            _ => {
                if d.abs() < 0.01 {
                    ctx.set_cursor_icon(self.choose_resize_cursor());
                    GrabType::Resize
                } else if d < 0.0 {
                    ctx.set_cursor_icon(CursorIcon::Grab);
                    GrabType::Move
                } else {
                    GrabType::None
                }
            }
        }
    }

    fn get_item_id_under_cursor(&self) -> Option<ItemId> {
        id_of_signed_distance(&self.instructions_for_id, self.cursor_from_pixels())
    }
}
