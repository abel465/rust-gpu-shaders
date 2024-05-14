use core::mem::MaybeUninit;

pub struct Stack<const N: usize, T> {
    pub buf: [MaybeUninit<T>; N],
    pub sp: usize,
}

impl<const N: usize, T> Stack<N, T>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            sp: 0,
        }
    }

    pub fn push(&mut self, x: T) {
        self.buf[self.sp] = MaybeUninit::new(x);
        self.sp += 1;
    }

    pub fn pop(&mut self) -> T {
        self.sp -= 1;
        unsafe { self.buf[self.sp].assume_init() }
    }

    pub fn peek(&self) -> T {
        unsafe { self.buf[self.sp - 1].assume_init() }
    }
}
