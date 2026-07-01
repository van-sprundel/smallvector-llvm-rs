#![deny(clippy::pedantic)]
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

impl<T: std::marker::Copy, const N: usize> Default for SmallVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::marker::Copy, const N: usize> SmallVec<T, N> {
    #[must_use]
    pub fn new() -> Self {
        Self::Inline {
            buf: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Inline { len, .. } => *len == 0,
            Self::Spilled(v) => v.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Spilled(v) => v.len(),
        }
    }
    pub fn capacity(&self) -> usize {
        match self {
            Self::Inline { .. } => N,
            Self::Spilled(v) => v.capacity(),
        }
    }

    pub fn push(&mut self, item: T) {
        match self {
            Self::Inline { buf, len } => {
                if *len < N {
                    // still fits on the stack
                    buf[*len] = MaybeUninit::new(item);
                    *len += 1;
                } else {
                    // spilled!
                    // move everything to the heap
                    let mut v = Vec::with_capacity(N * 2);
                    for item in buf.iter().take(*len) {
                        // SAFETY: elements 0..len are initialized
                        v.push(unsafe { item.assume_init() });
                    }
                    v.push(item);
                    *self = Self::Spilled(v);
                }
            }
            Self::Spilled(v) => v.push(item),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self {
            Self::Inline { buf, len } => {
                if *len == 0 {
                    None
                } else {
                    *len -= 1;
                    Some(unsafe { buf[*len].assume_init() })
                }
            }
            Self::Spilled(v) => v.pop(),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Inline { buf, len } => {
                // SAFETY: elements 0..len are initialized
                unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<T>(), *len) }
            }
            Self::Spilled(v) => v.as_slice(),
        }
    }
}
