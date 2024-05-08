use super::sdf_builder_tree::ItemId;
use core::ops::Neg;
use dfutils::{primitives_enum::Shape, sdf::Sdf};
use glam::Vec2;
use shared::{
    sdf_interpreter::{Instruction, Operator, Transform},
    stack::Stack,
};

pub fn id_of_signed_distance(instructions: &[InstructionForId], p: Vec2) -> Option<ItemId> {
    if instructions.is_empty() {
        return None;
    }
    let mut stack = Stack::<8, IdAndDistance>::new();
    for instruction in instructions {
        match instruction {
            InstructionForId::Operator(op, _) => {
                let b = stack.pop();
                let a = stack.pop();
                stack.push(operate_with_id(op, a, b));
            }
            InstructionForId::Shape(shape, id, Transform { position }) => {
                stack.push(IdAndDistance::new(
                    shape.signed_distance(p - *position),
                    *id,
                ));
            }
        }
    }
    Some(stack.pop().id)
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub enum InstructionForId {
    Operator(Operator, ItemId),
    Shape(Shape, ItemId, Transform),
}

impl Into<Instruction> for InstructionForId {
    fn into(self) -> Instruction {
        match self {
            InstructionForId::Operator(op, _) => Instruction::Operator(op),
            InstructionForId::Shape(shape, _, transform) => Instruction::Shape(shape, transform),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct IdAndDistance {
    d: f32,
    id: ItemId,
}

impl IdAndDistance {
    fn new(d: f32, id: ItemId) -> Self {
        Self { d, id }
    }
    fn min(self, other: Self) -> Self {
        if self.d < other.d {
            self
        } else {
            other
        }
    }
    fn max(self, other: Self) -> Self {
        if self.d > other.d {
            self
        } else {
            other
        }
    }
}

impl Neg for IdAndDistance {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            d: -self.d,
            id: self.id,
        }
    }
}

fn operate_with_id(op: &Operator, a: IdAndDistance, b: IdAndDistance) -> IdAndDistance {
    use Operator::*;
    match op {
        Union => a.min(b),
        Intersect => a.max(b),
        Subtract => b.max(-a),
        Xor => a.min(b).max(-a.max(b)),
    }
}
