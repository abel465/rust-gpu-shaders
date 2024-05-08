pub struct Stack<const N: usize, T> {
    pub buf: [T; N],
    pub sp: usize,
}

impl<const N: usize, T> Stack<N, T>
where
    T: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            buf: [T::default(); N],
            sp: 0,
        }
    }
    pub fn push(&mut self, x: T) {
        self.buf[self.sp] = x;
        self.sp += 1;
    }
    pub fn pop(&mut self) -> T {
        self.sp -= 1;
        self.buf[self.sp]
    }
    pub fn peek(&self) -> T {
        self.buf[self.sp - 1]
    }
}
