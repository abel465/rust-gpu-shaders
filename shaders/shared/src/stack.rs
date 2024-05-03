pub struct Stack<const N: usize> {
    pub buf: [f32; N],
    pub sp: usize,
}

impl<const N: usize> Stack<N> {
    pub fn new() -> Self {
        Self {
            buf: [0.0; N],
            sp: 0,
        }
    }
    pub fn push(&mut self, x: f32) {
        self.buf[self.sp] = x;
        self.sp += 1;
    }
    pub fn pop(&mut self) -> f32 {
        self.sp -= 1;
        self.buf[self.sp]
    }
    pub fn peek(&self) -> f32 {
        self.buf[self.sp - 1]
    }
}
