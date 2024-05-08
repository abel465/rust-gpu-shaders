use crate::stack::Stack;
use bytemuck::{Pod, Zeroable};
use spirv_std::glam::Vec2;
#[cfg_attr(not(target_arch = "spirv"), allow(unused_imports))]
use spirv_std::num_traits::Float;

#[cfg(not(target_arch = "spirv"))]
#[derive(Copy, Clone)]
pub enum OpCode0 {
    Push(f32),
    Pushx,
    Pushy,
    Sqrt,
    Square,
    Neg,
    Sin,
    Cos,
    Abs,
    Add,
    Sub,
    Mul,
    Div,
}

#[cfg_attr(not(target_arch = "spirv"), derive(strum::EnumIter, strum::Display))]
#[derive(Copy, Clone)]
#[repr(C)]
pub enum OpCode {
    Push,
    Pushx,
    Pushy,
    Sqrt,
    Square,
    Neg,
    Sin,
    Cos,
    Abs,
    Add,
    Sub,
    Mul,
    Div,
}

impl OpCode {
    pub fn from_u32(x: u32) -> Self {
        unsafe { core::mem::transmute(x) }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl From<OpCode0> for OpCodeStruct {
    fn from(op: OpCode0) -> Self {
        if let OpCode0::Push(value) = op {
            return Self {
                op: OpCode::Push as u32,
                value,
                pad1: 0,
                pad2: 0,
            };
        }

        Self {
            op: match op {
                OpCode0::Push(_) => OpCode::Push,
                OpCode0::Pushx => OpCode::Pushx,
                OpCode0::Pushy => OpCode::Pushy,
                OpCode0::Sqrt => OpCode::Sqrt,
                OpCode0::Square => OpCode::Square,
                OpCode0::Neg => OpCode::Neg,
                OpCode0::Sin => OpCode::Sin,
                OpCode0::Cos => OpCode::Cos,
                OpCode0::Abs => OpCode::Abs,
                OpCode0::Add => OpCode::Add,
                OpCode0::Sub => OpCode::Sub,
                OpCode0::Mul => OpCode::Mul,
                OpCode0::Div => OpCode::Div,
            } as u32,
            value: 0.0,
            pad1: 0,
            pad2: 0,
        }
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct OpCodeStruct {
    pub op: u32,
    pub value: f32,
    pub pad1: u32,
    pub pad2: u32,
}

pub struct Interpreter<const STACK_SIZE: usize> {
    stack: Stack<STACK_SIZE, f32>,
    p: Vec2,
}

impl<const STACK_SIZE: usize> Interpreter<STACK_SIZE> {
    pub fn new(p: Vec2) -> Self {
        Self {
            stack: Stack::<STACK_SIZE, f32>::new(),
            p,
        }
    }
    pub fn interpret(&mut self, ops: &[OpCodeStruct], n: usize) -> f32 {
        for i in 0..n {
            let ocs = ops[i];
            let op = OpCode::from_u32(ocs.op);
            let value = ocs.value;
            match op {
                OpCode::Push => {
                    self.stack.push(value);
                }
                OpCode::Pushx => {
                    self.stack.push(self.p.x);
                }
                OpCode::Pushy => {
                    self.stack.push(self.p.y);
                }
                OpCode::Sqrt => {
                    let a = self.stack.pop();
                    self.stack.push(a.sqrt());
                }
                OpCode::Square => {
                    let a = self.stack.pop();
                    self.stack.push(a * a);
                }
                OpCode::Neg => {
                    let a = self.stack.pop();
                    self.stack.push(-a);
                }
                OpCode::Sin => {
                    let a = self.stack.pop();
                    self.stack.push(a.sin());
                }
                OpCode::Cos => {
                    let a = self.stack.pop();
                    self.stack.push(a.cos());
                }
                OpCode::Abs => {
                    let a = self.stack.pop();
                    self.stack.push(a.abs());
                }
                OpCode::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a + b);
                }
                OpCode::Sub => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a - b);
                }
                OpCode::Mul => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a * b);
                }
                OpCode::Div => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(a / b);
                }
            }
        }
        self.stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spirv_std::glam::vec2;

    pub fn length() -> Vec<OpCode0> {
        use OpCode0::*;
        vec![Pushx, Square, Pushy, Square, Add, Sqrt]
    }
    pub fn disk(r: f32) -> Vec<OpCode0> {
        use OpCode0::*;
        let mut vec = length();
        vec.extend(vec![Push(r), Sub]);
        vec
    }

    #[test]
    fn test_disk() {
        const STACK_SIZE: usize = 8;
        let p = vec2(0.9, 0.2);
        let r = 0.3;
        let mut interpreter = Interpreter::<STACK_SIZE>::new(p);
        let ops: Vec<OpCodeStruct> = disk(r).iter().map(|op| (*op).into()).collect();
        let d = interpreter.interpret(&ops.as_slice(), ops.len());
        assert_eq!(d, crate::sdf_2d::disk(p, r))
    }
}
