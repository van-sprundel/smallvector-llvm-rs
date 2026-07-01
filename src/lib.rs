use std::mem::MaybeUninit;

pub enum SmallVec<T, const N: usize> {
    // fixed-size stack arr
    Inline {
        buf: [MaybeUninit<T>; N],
        len: usize,
    },
    // spilled heap alloc
    Spilled(Vec<T>),
}

impl<T: std::marker::Copy, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        Self::Inline {
            buf: [MaybeUninit::uninit(); N], 
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Spilled(v) => v.len(),
        }
    }
}
