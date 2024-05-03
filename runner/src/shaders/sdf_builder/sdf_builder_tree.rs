use super::shape_ui::ShapeUi;
use dfutils::primitives_enum::Shape;
use egui::NumExt as _;
use glam::Vec2;
use shared::stack::Stack;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, strum::EnumIter, strum::IntoStaticStr)]
pub enum Operator {
    Union,
    Intersection,
    Difference,
    Xor,
}

impl Operator {
    fn operate(&self, a: f32, b: f32) -> f32 {
        use Operator::*;
        match self {
            Union => a.min(b),
            Intersection => a.max(b),
            Difference => b.max(-a),
            Xor => a.min(b).max(-a.max(b)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Operator(Operator, Vec<ItemId>),
    Shape(Shape),
}

impl From<Shape> for Item {
    fn from(shape: Shape) -> Self {
        Item::Shape(shape)
    }
}

impl dfutils::sdf::Sdf for SdfInstructions {
    fn signed_distance(&self, p: Vec2) -> f32 {
        if self.instructions.is_empty() {
            return f32::INFINITY;
        }
        let mut stack = Stack::<8>::new();
        for bob in &self.instructions {
            match bob {
                Instruction::Operator(op) => {
                    let b = stack.pop();
                    let a = stack.pop();
                    stack.push(op.operate(a, b));
                }
                Instruction::Shape(shape) => {
                    stack.push(shape.signed_distance(p));
                }
            }
        }
        stack.pop()
    }
}

pub struct SdfInstructions {
    instructions: Vec<Instruction>,
}

impl SdfInstructions {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }
}

pub enum Instruction {
    Operator(Operator),
    Shape(Shape),
}

impl SdfBuilderTree {
    pub fn generate_instructions(&self) -> Vec<Instruction> {
        let mut instructions = vec![];
        self.generate_instructions_impl(&self.root_id, &mut instructions);
        instructions
    }

    fn generate_instructions_impl(&self, id: &ItemId, instructions: &mut Vec<Instruction>) -> bool {
        if let Some(item) = self.items.get(id) {
            match item {
                Item::Operator(op, ids) => {
                    let mut items = ids.iter().rev();
                    let mut r1 = false;
                    while !r1 {
                        if let Some(next_id) = items.next() {
                            r1 = self.generate_instructions_impl(next_id, instructions);
                        } else {
                            return false;
                        }
                    }
                    let mut r2 = false;
                    while !r2 {
                        if let Some(next_id) = items.next() {
                            r2 = self.generate_instructions_impl(next_id, instructions);
                        } else {
                            return true;
                        }
                    }
                    instructions.push(Instruction::Operator(*op));
                    for next_id in items {
                        if self.generate_instructions_impl(next_id, instructions) {
                            instructions.push(Instruction::Operator(*op));
                        }
                    }
                    true
                }
                Item::Shape(shape) => {
                    instructions.push(Instruction::Shape(*shape));
                    true
                }
            }
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub enum Command {
    /// Set the selected item
    SetSelectedItem(Option<ItemId>, Option<Item>),

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
    pub selected_item: (Option<ItemId>, Option<Item>),

    /// If a drag is ongoing, this is the id of the destination container (if any was identified)
    ///
    /// This is used to highlight the target container.
    target_container: Option<ItemId>,

    /// Channel to receive commands from the UI
    command_receiver: std::sync::mpsc::Receiver<Command>,

    /// Channel to send commands from the UI
    pub command_sender: std::sync::mpsc::Sender<Command>,

    pub grid_needs_updating: bool,
}

impl Default for SdfBuilderTree {
    fn default() -> Self {
        let root_item = Item::Operator(Operator::Union, Vec::new());
        let root_id = ItemId::new();

        let (command_sender, command_receiver) = std::sync::mpsc::channel();

        let mut res = Self {
            items: std::iter::once((root_id, root_item)).collect(),
            root_id,
            selected_item: (None, None),
            target_container: None,
            command_receiver,
            command_sender,
            grid_needs_updating: true,
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
        self.add_leaf(self.root_id, Shape::Disk(Default::default()));
    }

    // pub fn get_selected_item(&self) -> Option<&Item> {
    //     self.selected_item.and_then(|id| self.items.get(&id))
    // }

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
        if let Some(id) = self.selected_item.0 {
            if id == item_id {
                self.selected_item.1 = Some(item);
            }
        }
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
            Item::Shape(_) => {}
        }
        if let Some((id, pos)) = self.parent_and_pos(item_id) {
            match self.items.get_mut(&id).unwrap() {
                Item::Operator(_, items) => {
                    items.remove(pos);
                }
                Item::Shape(_) => {}
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

    // fn add_container(&mut self, parent_id: ItemId, operator: Operator) -> ItemId {
    //     let id = ItemId::new();
    //     let item = Item::Operator(operator, Vec::new());
    //
    //     self.items.insert(id, item);
    //
    //     if let Some(Item::Operator(_, children)) = self.items.get_mut(&parent_id) {
    //         children.push(id);
    //     }
    //
    //     id
    // }

    fn add_leaf(&mut self, parent_id: ItemId, shape: Shape) {
        let id = ItemId::new();
        let item = Item::Shape(shape);

        self.items.insert(id, item);

        if let Some(Item::Operator(_, children)) = self.items.get_mut(&parent_id) {
            children.push(id);
        }
    }

    fn send_command(&self, command: Command) {
        // The only way this can fail is if the receiver has been dropped.
        self.command_sender.send(command).ok();
    }
}

//
// UI stuff
//
impl SdfBuilderTree {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        for shape in Shape::iter() {
            let item0 = Item::Shape(shape);
            let label: &str = shape.into();
            let response = egui::Frame::none()
                .stroke(egui::Stroke {
                    width: 4.0,
                    color: egui::Color32::DARK_GRAY,
                })
                .inner_margin(egui::Margin::same(5.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(label)
                            .selectable(false)
                            .sense(egui::Sense::click_and_drag()),
                    )
                })
                .inner;
            self.handle_drag_and_drop_interaction(
                ui,
                ItemId::new(),
                false,
                &response,
                None,
                Some(item0),
            );
        }
        ui.separator();
        for operator in Operator::iter() {
            let item0 = Item::Operator(operator, Vec::new());
            let label: &str = operator.into();
            let response = ui.add(
                egui::Label::new(label)
                    .selectable(false)
                    .sense(egui::Sense::click_and_drag()),
            );
            self.handle_drag_and_drop_interaction(
                ui,
                ItemId::new(),
                false,
                &response,
                None,
                Some(item0),
            );
        }
        ui.separator();

        if let Some(top_level_items) = self.container(self.root_id) {
            self.container_children_ui(ui, top_level_items);
        }

        // deselect by clicking in the empty space
        if ui
            .interact(
                ui.available_rect_before_wrap(),
                "empty_space".into(),
                egui::Sense::click(),
            )
            .clicked()
        {
            self.send_command(Command::SetSelectedItem(None, None));
        }

        // always reset the target container
        self.target_container = None;

        while let Ok(command) = self.command_receiver.try_recv() {
            println!("Received command: {command:?}");
            match command {
                Command::SetSelectedItem(item_id, item) => self.selected_item = (item_id, item),
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
                let ret = ui.add(
                    egui::Label::new(format!("{operator:?}"))
                        .selectable(false)
                        .sense(egui::Sense::click_and_drag()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("x").clicked() {
                        self.send_command(Command::RemoveItem { item_id });
                    };
                });
                ret
            })
            .body(|ui| {
                self.container_children_ui(ui, children);
            });

        if head_response.inner.clicked() {
            self.send_command(Command::SetSelectedItem(Some(item_id), None));
        }

        if self.target_container == Some(item_id) {
            ui.painter().rect_stroke(
                head_response.inner.rect,
                2.0,
                (1.0, ui.visuals().selection.bg_fill),
            );
        }

        self.handle_drag_and_drop_interaction(
            ui,
            item_id,
            true,
            &head_response.inner.union(response),
            body_resp.as_ref().map(|r| &r.response),
            None,
        );
    }

    fn container_children_ui(&self, ui: &mut egui::Ui, children: &Vec<ItemId>) {
        for child_id in children {
            // check if the item is selected
            ui.visuals_mut().override_text_color = if Some(*child_id) == self.selected_item.0 {
                Some(ui.visuals().selection.bg_fill)
            } else {
                None
            };

            match self.items.get(child_id) {
                Some(Item::Operator(operator, children)) => {
                    self.container_ui(ui, *child_id, operator, children);
                }
                Some(Item::Shape(shape)) => {
                    self.leaf_ui(ui, *child_id, *shape);
                }
                None => {}
            }
        }
    }

    fn leaf_ui(&self, ui: &mut egui::Ui, item_id: ItemId, shape: Shape) {
        let response = egui::Frame::none()
            .stroke(egui::Stroke {
                width: 4.0,
                color: egui::Color32::DARK_GRAY,
            })
            .inner_margin(egui::Margin::same(5.0))
            .show(ui, |ui| {
                let label: &str = shape.into();
                let ret = ui.horizontal(|ui| {
                    let ret = ui.add(
                        egui::Label::new(label)
                            .selectable(false)
                            .sense(egui::Sense::click_and_drag()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("x").clicked() {
                            self.send_command(Command::RemoveItem { item_id });
                        };
                    });
                    ret
                });
                egui::Grid::new("shape_params_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        let new_shape = shape.ui(ui);
                        if shape != new_shape {
                            self.send_command(Command::EditItem {
                                item: new_shape.into(),
                                item_id,
                            });
                        }
                    });
                ret
            })
            .inner
            .inner;

        if response.clicked() {
            self.send_command(Command::SetSelectedItem(Some(item_id), Some(shape.into())));
        }

        self.handle_drag_and_drop_interaction(ui, item_id, false, &response, None, None);
    }

    fn handle_drag_and_drop_interaction(
        &self,
        ui: &egui::Ui,
        item_id: ItemId,
        is_container: bool,
        response: &egui::Response,
        body_response: Option<&egui::Response>,
        new_item: Option<Item>,
    ) {
        //
        // handle start of drag
        //

        if response.drag_started() {
            egui::DragAndDrop::set_payload(ui.ctx(), item_id);

            // force selection to the dragged item
            self.send_command(Command::SetSelectedItem(Some(item_id), new_item.clone()));
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
                (2.0, egui::Color32::BLACK),
            );

            // note: can't use `response.drag_released()` because we not the item which
            // started the drag
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(item) = &self.selected_item.1 {
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
}
