use crate::stack::Stack;
use dfutils::primitives_enum::Shape;
use spirv_std::glam::Vec2;

#[cfg_attr(
    not(target_arch = "spirv"),
    derive(strum::EnumIter, strum::IntoStaticStr)
)]
#[derive(Clone, Copy, Debug)]
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

pub struct SdfInstructions<'a> {
    instructions: &'a [Instruction],
}

impl<'a> SdfInstructions<'a> {
    pub fn new(instructions: &'a [Instruction]) -> Self {
        Self { instructions }
    }
}

pub enum Instruction {
    Operator(Operator),
    Shape(Shape),
}

impl<'a> dfutils::sdf::Sdf for SdfInstructions<'a> {
    fn signed_distance(&self, p: Vec2) -> f32 {
        if self.instructions.is_empty() {
            return f32::INFINITY;
        }
        let mut stack = Stack::<8>::new();
        for instruction in self.instructions {
            match instruction {
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
