use super::instructions::InstructionForId;
use super::{icons::TextureHandles, shape_ui::ShapeUi};
use dfutils::primitives_enum::Shape;
use egui::{load::SizedTexture, NumExt as _, TextureHandle};
use egui_winit::winit::dpi::PhysicalSize;
use glam::*;
use itertools::izip;
use shared::{
    from_pixels,
    sdf_interpreter::{Instruction, Operator, Transform},
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemId(u32);

impl ItemId {
    fn new() -> Self {
        Self(rand::random())
    }
}

impl std::fmt::Debug for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:04x}", self.0)
    }
}

impl From<ItemId> for egui::Id {
    fn from(id: ItemId) -> Self {
        Self::new(id)
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Operator(Operator, Vec<ItemId>),
    Shape(Shape, Transform),
}

impl From<Shape> for Item {
    fn from(shape: Shape) -> Self {
        Item::Shape(shape, Default::default())
    }
}

impl From<Operator> for Item {
    fn from(op: Operator) -> Self {
        Item::Operator(op, Default::default())
    }
}

#[derive(Debug)]
pub struct SelectedItem {
    pub id: Option<ItemId>,
    pub new_item: Option<Item>,
}

impl From<ItemId> for SelectedItem {
    fn from(id: ItemId) -> Self {
        Self {
            id: Some(id),
            new_item: None,
        }
    }
}

impl SelectedItem {
    pub const NONE: Self = SelectedItem {
        id: None,
        new_item: None,
    };
    const fn new(id: ItemId, item: Item) -> Self {
        SelectedItem {
            id: Some(id),
            new_item: Some(item),
        }
    }
}

#[derive(Debug)]
pub enum Command {
    /// Set the selected item
    SetSelectedItem(SelectedItem),

    /// Move the currently dragged item to the given container and position.
    MoveItem {
        moved_item_id: ItemId,
        target_container_id: ItemId,
        target_position_index: usize,
    },

    /// Add the currently dragged item to the given container and position.
    AddItem {
        item: Item,
        new_item_id: ItemId,
        target_container_id: ItemId,
        target_position_index: usize,
    },

    /// Edit the selected item.
    EditItem { item: Item, item_id: ItemId },

    /// Remove the selected item.
    RemoveItem { item_id: ItemId },

    /// Specify the currently identified target container to be highlighted.
    HighlightTargetContainer(ItemId),
}

pub struct SdfBuilderTree {
    /// All items
    items: HashMap<ItemId, Item>,

    /// Id of the root item (not displayed in the UI)
    root_id: ItemId,

    /// Selected item, if any
    pub selected_item: SelectedItem,

    /// If a drag is ongoing, this is the id of the destination container (if any was identified)
    ///
    /// This is used to highlight the target container.
    target_container: Option<ItemId>,

    /// Channel to receive commands from the UI
    command_receiver: std::sync::mpsc::Receiver<Command>,

    /// Channel to send commands from the UI
    command_sender: std::sync::mpsc::Sender<Command>,

    pub grid_needs_updating: bool,

    extra_item: Option<(Shape, Transform)>,

    operator_mode: Operator,
}

impl Default for SdfBuilderTree {
    fn default() -> Self {
        let root_item = Item::Operator(Operator::Union, Vec::new());
        let root_id = ItemId::new();

        let (command_sender, command_receiver) = std::sync::mpsc::channel();

        let mut res = Self {
            items: std::iter::once((root_id, root_item)).collect(),
            root_id,
            selected_item: SelectedItem::NONE,
            target_container: None,
            command_receiver,
            command_sender,
            grid_needs_updating: true,
            extra_item: None,
            operator_mode: Operator::Union,
        };

        res.populate();

        res
    }
}

//
// Data stuff
//
impl SdfBuilderTree {
    fn populate(&mut self) {
        // self.add_leaf(self.root_id, Shape::Rectangle(Default::default()));
    }

    pub fn get_selected_item(&self) -> Option<&Item> {
        self.selected_item.id.and_then(|id| self.items.get(&id))
    }

    fn container(&self, id: ItemId) -> Option<&Vec<ItemId>> {
        match self.items.get(&id) {
            Some(Item::Operator(_, children)) => Some(children),
            _ => None,
        }
    }

    /// Does some container contain the given item?
    ///
    /// Used to test if a target location is suitable for a given dragged item.
    fn contains(&self, container_id: ItemId, item_id: ItemId) -> bool {
        if let Some(children) = self.container(container_id) {
            if container_id == item_id {
                return true;
            }

            if children.contains(&item_id) {
                return true;
            }

            for child_id in children {
                if self.contains(*child_id, item_id) {
                    return true;
                }
            }

            return false;
        }

        false
    }

    /// Move item `item_id` to `container_id` at position `pos`.
    fn move_item(&mut self, item_id: ItemId, container_id: ItemId, mut pos: usize) {
        println!("Moving {item_id:?} to {container_id:?} at position {pos:?}");

        // Remove the item from its current location. Note: we must adjust the target position if the item is
        // moved within the same container, as the removal might shift the positions by one.
        if let Some((source_parent_id, source_pos)) = self.parent_and_pos(item_id) {
            if let Some(Item::Operator(_, children)) = self.items.get_mut(&source_parent_id) {
                children.remove(source_pos);
            }

            if source_parent_id == container_id && source_pos < pos {
                pos -= 1;
            }
        }

        if let Some(Item::Operator(_, children)) = self.items.get_mut(&container_id) {
            children.insert(pos.at_most(children.len()), item_id);
        }
    }

    /// Add item `item_id` to `container_id` at position `pos`.
    fn add_item(&mut self, item: Item, item_id: ItemId, container_id: ItemId, pos: usize) {
        println!("Adding {item_id:?} to {container_id:?} at position {pos:?}");

        self.items.insert(item_id, item);

        if let Some(Item::Operator(_, children)) = self.items.get_mut(&container_id) {
            children.insert(pos.at_most(children.len()), item_id);
        }
    }

    /// Edit item `item_id`.
    fn edit_item(&mut self, item: Item, item_id: ItemId) {
        println!("Editing {item_id:?}");

        *self.items.get_mut(&item_id).unwrap() = item.clone();
    }

    /// Remove item `item_id`.
    fn remove_item(&mut self, item_id: ItemId) {
        println!("Removing {item_id:?}");

        let item = self.items.get(&item_id).unwrap();
        match item {
            Item::Operator(_, items) => {
                for id in items {
                    self.send_command(Command::RemoveItem { item_id: *id })
                }
            }
            Item::Shape(_, _) => {}
        }
        if let Some((id, pos)) = self.parent_and_pos(item_id) {
            match self.items.get_mut(&id).unwrap() {
                Item::Operator(_, items) => {
                    items.remove(pos);
                }
                Item::Shape(_, _) => {}
            }
        }
        self.items.remove(&item_id);
    }

    /// Find the parent of an item, and the index of that item within the parent's children.
    fn parent_and_pos(&self, id: ItemId) -> Option<(ItemId, usize)> {
        if id == self.root_id {
            None
        } else {
            self.parent_and_pos_impl(id, self.root_id)
        }
    }

    fn parent_and_pos_impl(&self, id: ItemId, container_id: ItemId) -> Option<(ItemId, usize)> {
        if let Some(children) = self.container(container_id) {
            for (idx, child_id) in children.iter().enumerate() {
                if child_id == &id {
                    return Some((container_id, idx));
                } else if self.container(*child_id).is_some() {
                    let res = self.parent_and_pos_impl(id, *child_id);
                    if res.is_some() {
                        return res;
                    }
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    fn add_leaf(&mut self, parent_id: ItemId, shape: Shape) {
        let id = ItemId::new();

        self.items.insert(id, shape.into());

        if let Some(Item::Operator(_, children)) = self.items.get_mut(&parent_id) {
            children.push(id);
        }
    }

    pub fn send_command(&self, command: Command) {
        // The only way this can fail is if the receiver has been dropped.
        self.command_sender.send(command).ok();
    }
}

//
// UI stuff
//
impl SdfBuilderTree {
    pub fn ui(&mut self, ui: &mut egui::Ui, icons: &TextureHandles, size: PhysicalSize<u32>) {
        self.shapes_ui(ui, &icons.shapes);
        ui.separator();
        self.operators_ui(ui, &icons.operators);
        ui.separator();

        if let Some(top_level_items) = self.container(self.root_id) {
            if top_level_items.is_empty() {
                self.root_drop_target(ui);
            } else {
                self.container_children_ui(ui, top_level_items);
            }
        }

        self.handle_extra_item(ui, size);

        // deselect by clicking in the empty space
        if ui
            .interact(
                ui.available_rect_before_wrap(),
                "empty_space".into(),
                egui::Sense::click(),
            )
            .clicked()
        {
            self.send_command(Command::SetSelectedItem(SelectedItem::NONE));
        }

        // always reset the target container
        self.target_container = None;

        while let Ok(command) = self.command_receiver.try_recv() {
            println!("Received command: {command:?}");
            match command {
                Command::SetSelectedItem(selected_item) => self.selected_item = selected_item,
                Command::MoveItem {
                    moved_item_id,
                    target_container_id,
                    target_position_index,
                } => {
                    self.move_item(moved_item_id, target_container_id, target_position_index);
                    self.grid_needs_updating = true;
                }
                Command::AddItem {
                    item,
                    new_item_id,
                    target_container_id,
                    target_position_index,
                } => {
                    self.add_item(
                        item,
                        new_item_id,
                        target_container_id,
                        target_position_index,
                    );
                    self.grid_needs_updating = true;
                }
                Command::EditItem { item, item_id } => {
                    self.edit_item(item, item_id);
                    self.grid_needs_updating = true;
                }
                Command::RemoveItem { item_id } => {
                    self.remove_item(item_id);
                    self.grid_needs_updating = true;
                }
                Command::HighlightTargetContainer(item_id) => {
                    self.target_container = Some(item_id);
                }
            }
        }
    }

    fn shapes_ui(&self, ui: &mut egui::Ui, icons: &[TextureHandle]) {
        let add_contents = |ui: &mut egui::Ui| {
            for (shape, icon, end_row) in
                izip!(Shape::iter(), icons, [false, true].into_iter().cycle())
            {
                use convert_case::{Case, Casing};
                let label = Into::<&str>::into(shape).to_case(Case::Title);
                let mut frame = egui::Frame::none()
                    .inner_margin(egui::Margin::same(3.0))
                    .begin(ui);
                let response = frame
                    .content_ui
                    .vertical_centered(|ui| {
                        let rect = ui
                            .label(&label)
                            .rect
                            .union(ui.image(SizedTexture::from_handle(icon)).rect);
                        ui.interact(rect, egui::Id::new(label), egui::Sense::click_and_drag())
                    })
                    .inner;
                if response.hovered() {
                    frame.frame.stroke = egui::Stroke::new(1.0, egui::Color32::DARK_GRAY);
                }
                frame.end(ui);
                self.handle_new_item_drag(ui, &response, shape.into());
                if end_row {
                    ui.end_row();
                }
            }
        };
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                egui::Grid::new("shape_icons_grid").show(ui, add_contents);
            });
    }

    fn operators_ui(&mut self, ui: &mut egui::Ui, icons: &[TextureHandle]) {
        egui::Grid::new("operator_icons_grid").show(ui, |ui| {
            for (operator, icon, end_row) in
                izip!(Operator::iter(), icons, [false, true].into_iter().cycle())
            {
                let label: &str = operator.into();
                let mut frame = egui::Frame::none()
                    .inner_margin(egui::Margin::same(3.0))
                    .begin(ui);
                let response = frame
                    .content_ui
                    .vertical_centered(|ui| {
                        let rect = ui
                            .label(label)
                            .rect
                            .union(ui.image(SizedTexture::from_handle(icon)).rect);
                        ui.interact(rect, egui::Id::new(label), egui::Sense::click_and_drag())
                    })
                    .inner;
                if response.clicked() {
                    self.operator_mode = operator;
                }
                frame.frame.stroke = if self.operator_mode == operator {
                    egui::Stroke::new(2.0, egui::Color32::LIGHT_RED)
                } else if response.hovered() {
                    egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
                } else {
                    egui::Stroke::NONE
                };
                frame.end(ui);
                self.handle_new_item_drag(ui, &response, operator.into());
                if end_row {
                    ui.end_row();
                }
            }
        });
    }

    fn container_ui(
        &self,
        ui: &mut egui::Ui,
        item_id: ItemId,
        operator: &Operator,
        children: &Vec<ItemId>,
    ) {
        let (response, head_response, body_resp) =
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                item_id.into(),
                true,
            )
            .show_header(ui, |ui| {
                let resp = ui.add(
                    egui::Label::new(format!("{operator:?}"))
                        .selectable(false)
                        .sense(egui::Sense::click_and_drag()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    let resp = ui.button("x");
                    if resp.clicked() {
                        self.send_command(Command::RemoveItem { item_id });
                    }
                    resp
                })
                .inner
                .union(resp)
            })
            .body(|ui| {
                self.container_children_ui(ui, children);
            });

        if head_response.inner.clicked() {
            self.send_command(Command::SetSelectedItem(item_id.into()));
        }

        if self.target_container == Some(item_id) {
            ui.painter().rect_stroke(
                head_response.inner.rect,
                2.0,
                (1.0, ui.visuals().selection.bg_fill),
            );
        }

        let mut response = head_response.inner.union(response);
        if children.is_empty() {
            if let Some(resp) = &body_resp {
                response.rect = response.rect.union(resp.response.rect.clone());
            }
        }

        self.handle_drag_and_drop_interaction(
            ui,
            item_id,
            true,
            &response,
            body_resp.as_ref().map(|r| &r.response),
        );
    }

    fn container_children_ui(&self, ui: &mut egui::Ui, children: &Vec<ItemId>) {
        for child_id in children {
            // check if the item is selected
            ui.visuals_mut().override_text_color = if Some(*child_id) == self.selected_item.id {
                Some(ui.visuals().selection.bg_fill)
            } else {
                None
            };

            match self.items.get(child_id) {
                Some(Item::Operator(operator, children)) => {
                    self.container_ui(ui, *child_id, operator, children);
                }
                Some(Item::Shape(shape, transform)) => {
                    self.leaf_ui(ui, *child_id, *shape, *transform);
                }
                None => {}
            }
        }
    }

    fn leaf_ui(&self, ui: &mut egui::Ui, item_id: ItemId, shape: Shape, transform: Transform) {
        let (response, head_response, body_resp) =
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                item_id.into(),
                false,
            )
            .show_header(ui, |ui| {
                let label: &str = shape.into();
                let resp = ui.add(
                    egui::Label::new(label)
                        .selectable(false)
                        .sense(egui::Sense::click_and_drag()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    let resp = ui.button("x");
                    if resp.clicked() {
                        self.send_command(Command::RemoveItem { item_id });
                    }
                    resp
                })
                .inner
                .union(resp)
            })
            .body(|ui| {
                egui::Grid::new("shape_params_grid").show(ui, |ui| {
                    let new_shape = shape.ui(ui);
                    let mut new_transform = transform;
                    ui.end_row();
                    ui.label("pos");
                    ui.add(egui::DragValue::new(&mut new_transform.position.x).speed(0.01));
                    ui.add(egui::DragValue::new(&mut new_transform.position.y).speed(0.01));
                    if shape != new_shape || transform != new_transform {
                        self.send_command(Command::EditItem {
                            item: Item::Shape(new_shape, new_transform),
                            item_id,
                        });
                    }
                });
            });

        if head_response.inner.clicked() {
            self.send_command(Command::SetSelectedItem(item_id.into()));
        }

        let mut response = head_response.inner.union(response);
        if let Some(resp) = body_resp {
            response = response.union(resp.response);
        }

        self.handle_drag_and_drop_interaction(ui, item_id, false, &response, None);
    }

    fn handle_new_item_drag(&self, ui: &egui::Ui, response: &egui::Response, new_item: Item) {
        if response.drag_started() {
            let item_id = ItemId::new();
            egui::DragAndDrop::set_payload(ui.ctx(), item_id);

            self.send_command(Command::SetSelectedItem(SelectedItem::new(
                item_id, new_item,
            )));
        }
    }

    fn handle_drag_and_drop_interaction(
        &self,
        ui: &egui::Ui,
        item_id: ItemId,
        is_container: bool,
        response: &egui::Response,
        body_response: Option<&egui::Response>,
    ) {
        //
        // handle start of drag
        //

        if response.drag_started() {
            egui::DragAndDrop::set_payload(ui.ctx(), item_id);

            // force selection to the dragged item
            self.send_command(Command::SetSelectedItem(item_id.into()));
        }

        //
        // handle candidate drop
        //

        // find the item being dragged
        let Some(dragged_item_id) = egui::DragAndDrop::payload(ui.ctx()).map(|payload| (*payload))
        else {
            // nothing is being dragged, we're done here
            return;
        };

        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

        let Some((parent_id, position_index_in_parent)) = self.parent_and_pos(item_id) else {
            // this shouldn't happen
            return;
        };

        let previous_container_id = if position_index_in_parent > 0 {
            self.container(parent_id)
                .map(|c| c[position_index_in_parent - 1])
                .filter(|id| self.container(*id).is_some())
        } else {
            None
        };

        let item_desc = crate::egui_components::drag_and_drop::DropItemDescription {
            id: item_id,
            is_container,
            parent_id,
            position_index_in_parent,
            previous_container_id,
        };

        //
        // compute the drag target areas based on the item and body responses
        //

        // adjust the drop target to account for the spacing between items
        let item_rect = response
            .rect
            .expand2(egui::Vec2::new(0.0, ui.spacing().item_spacing.y / 2.0));
        let body_rect = body_response.map(|r| {
            r.rect
                .expand2(egui::Vec2::new(0.0, ui.spacing().item_spacing.y))
        });

        //
        // find the candidate drop target
        //

        let drop_target = crate::egui_components::drag_and_drop::find_drop_target(
            ui,
            &item_desc,
            item_rect,
            body_rect,
            response.rect.height(),
        );

        if let Some(drop_target) = drop_target {
            // We cannot allow the target location to be "inside" the dragged item, because that would amount moving
            // myself inside of me.

            if self.contains(dragged_item_id, drop_target.target_parent_id) {
                return;
            }

            // extend the cursor to the right of the enclosing container
            let mut span_x = drop_target.indicator_span_x;
            span_x.max = ui.cursor().right();

            ui.painter().hline(
                span_x,
                drop_target.indicator_position_y,
                (4.0, egui::Color32::BLACK),
            );

            // note: can't use `response.drag_released()` because we not the item which
            // started the drag
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(item) = &self.selected_item.new_item {
                    self.send_command(Command::AddItem {
                        item: item.clone(),
                        new_item_id: dragged_item_id,
                        target_container_id: drop_target.target_parent_id,
                        target_position_index: drop_target.target_position_index,
                    });
                } else {
                    self.send_command(Command::MoveItem {
                        moved_item_id: dragged_item_id,
                        target_container_id: drop_target.target_parent_id,
                        target_position_index: drop_target.target_position_index,
                    });
                }

                egui::DragAndDrop::clear_payload(ui.ctx());
            } else {
                self.send_command(Command::HighlightTargetContainer(
                    drop_target.target_parent_id,
                ));
            }
        }
    }

    fn root_drop_target(&self, ui: &mut egui::Ui) {
        let rect = ui.allocate_space(egui::vec2(ui.available_width(), 12.0)).1;

        // find the item being dragged
        let Some(dragged_item_id) = egui::DragAndDrop::payload(ui.ctx()).map(|payload| (*payload))
        else {
            // nothing is being dragged, we're done here
            return;
        };

        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

        if ui.rect_contains_pointer(rect) {
            ui.painter()
                .hline(rect.x_range(), rect.bottom(), (4.0, egui::Color32::BLACK));

            // note: can't use `response.drag_released()` because we not the item which
            // started the drag
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(item) = &self.selected_item.new_item {
                    self.send_command(Command::AddItem {
                        item: item.clone(),
                        new_item_id: dragged_item_id,
                        target_container_id: self.root_id,
                        target_position_index: 0,
                    });
                }

                egui::DragAndDrop::clear_payload(ui.ctx());
            } else {
                self.send_command(Command::HighlightTargetContainer(self.root_id));
            }
        }
    }
}

//
// Instruction generation
//
impl SdfBuilderTree {
    pub fn generate_instructions(&self) -> (Vec<Instruction>, Vec<InstructionForId>) {
        let capacity = self.items.len() + self.extra_item.is_some() as usize;
        let mut instructions_for_id = Vec::with_capacity(capacity);
        self.generate_instructions_for_id(&self.root_id, &mut instructions_for_id);
        let mut instructions: Vec<_> = instructions_for_id
            .iter()
            .map(|instruction_for_id| (*instruction_for_id).into())
            .collect();
        if let Some((shape, transform)) = self.extra_item {
            if instructions.is_empty() {
                instructions.push(Instruction::Shape(shape, transform));
            } else {
                instructions.insert(0, Instruction::Shape(shape, transform));
                instructions.push(Instruction::Operator(self.operator_mode));
            }
        }
        (instructions, instructions_for_id)
    }

    fn generate_instructions_for_id(
        &self,
        id: &ItemId,
        instructions: &mut Vec<InstructionForId>,
    ) -> bool {
        if let Some(item) = self.items.get(id) {
            match item {
                Item::Operator(op, ids) => {
                    let mut items = ids.iter().rev();
                    let mut r1 = false;
                    while !r1 {
                        if let Some(next_id) = items.next() {
                            r1 = self.generate_instructions_for_id(next_id, instructions);
                        } else {
                            return false;
                        }
                    }
                    let mut r2 = false;
                    while !r2 {
                        if let Some(next_id) = items.next() {
                            r2 = self.generate_instructions_for_id(next_id, instructions);
                        } else {
                            return true;
                        }
                    }
                    instructions.push(InstructionForId::Operator(*op, *id));
                    for next_id in items {
                        if self.generate_instructions_for_id(next_id, instructions) {
                            instructions.push(InstructionForId::Operator(*op, *id));
                        }
                    }
                    true
                }
                Item::Shape(shape, transform) => {
                    instructions.push(InstructionForId::Shape(*shape, *id, *transform));
                    true
                }
            }
        } else {
            false
        }
    }

    fn handle_extra_item(&mut self, ui: &mut egui::Ui, size: PhysicalSize<u32>) {
        let extra_item =
            if !ui.ui_contains_pointer() && egui::DragAndDrop::has_any_payload(ui.ctx()) {
                if let Some(Item::Shape(shape, _)) = self.selected_item.new_item {
                    ui.input(|i| i.pointer.latest_pos()).and_then(|pos| {
                        let transform = Transform {
                            position: from_pixels(vec2(pos.x, pos.y), size.into()),
                        };
                        Some((shape, transform))
                    })
                } else {
                    None
                }
            } else {
                None
            };
        if ui.input(|i| i.pointer.primary_released()) {
            if let Some((shape, transform)) = self.extra_item {
                match self.operator_mode {
                    Operator::Union => {
                        self.add_item(
                            Item::Shape(shape, transform),
                            self.selected_item.id.unwrap(),
                            self.root_id,
                            0,
                        );
                    }
                    _ => {
                        if self.items.len() <= 1 {
                            self.add_item(
                                Item::Shape(shape, transform),
                                self.selected_item.id.unwrap(),
                                self.root_id,
                                0,
                            );
                        } else {
                            let container_op_id = ItemId::new();
                            let container_union_id = ItemId::new();
                            let item_id = self.selected_item.id.unwrap();
                            let current_root_items = if let Some(Item::Operator(_, children)) =
                                self.items.get_mut(&self.root_id)
                            {
                                let copy = children.clone();
                                *children = vec![container_op_id];
                                copy
                            } else {
                                return;
                            };
                            self.items.insert(
                                container_op_id,
                                Item::Operator(
                                    self.operator_mode,
                                    vec![container_union_id, item_id],
                                ),
                            );
                            self.items.insert(
                                container_union_id,
                                Item::Operator(Operator::Union, current_root_items),
                            );
                            self.items.insert(item_id, Item::Shape(shape, transform));
                        }
                    }
                }
            }
        }
        if extra_item != self.extra_item {
            self.extra_item = extra_item;
            self.grid_needs_updating = true;
        }
    }
}
