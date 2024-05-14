use crate::stack::Stack;
use dfutils::sdf::*;
use spirv_std::glam::Vec2;

#[cfg_attr(
    not(target_arch = "spirv"),
    derive(Debug, strum::EnumIter, strum::IntoStaticStr)
)]
#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    Union,
    Intersect,
    Subtract,
    Xor,
}

impl Operator {
    fn operate<T>(&self, a: T, b: T) -> T
    where
        T: Copy + SignedDistance,
    {
        use Operator::*;
        match self {
            Union => a.union(&b),
            Intersect => a.intersect(&b),
            Subtract => a.subtract(&b),
            Xor => a.xor(&b),
        }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Transform {
    pub position: Vec2,
}

pub struct SdfInstructions<'a, U: SignedDistance, T: Copy + Sdf<T = U>> {
    instructions: &'a [Instruction<T>],
}

impl<'a, U: SignedDistance, T: Copy + Sdf<T = U>> SdfInstructions<'a, U, T> {
    pub fn new(instructions: &'a [Instruction<T>]) -> Self {
        Self { instructions }
    }
}

pub enum Instruction<T: Copy> {
    Operator(Operator),
    Sdf(T, Transform),
}

impl<'a, U, T> Sdf for SdfInstructions<'a, U, T>
where
    U: SignedDistance,
    T: Clone + Copy + Sdf<T = U>,
{
    type T = U;
    fn signed_distance(&self, p: Vec2) -> U {
        if self.instructions.is_empty() {
            return U::divergent();
        }
        let mut stack = Stack::<8, U>::new();
        for instruction in self.instructions {
            match instruction {
                Instruction::Operator(op) => {
                    let b = stack.pop();
                    let a = stack.pop();
                    stack.push(op.operate(a, b));
                }
                Instruction::Sdf(sdf, Transform { position }) => {
                    stack.push(sdf.signed_distance(p - *position));
                }
            }
        }
        stack.pop()
    }
}
