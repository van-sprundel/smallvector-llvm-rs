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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let sv = SmallVec::<i32, 4>::new();
        assert!(sv.is_empty());
        assert_eq!(sv.len(), 0);
        assert_eq!(sv.capacity(), 4);
    }

    #[test]
    fn push_stays_inline() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(10);
        sv.push(20);
        assert_eq!(sv.len(), 2);
        assert!(matches!(sv, SmallVec::Inline { .. }));
        assert_eq!(sv.as_slice(), &[10, 20]);
    }

    #[test]
    fn push_fills_inline_exactly() {
        let mut sv = SmallVec::<i32, 3>::new();
        sv.push(1);
        sv.push(2);
        sv.push(3);
        assert_eq!(sv.len(), 3);
        assert!(matches!(sv, SmallVec::Inline { .. }));
        assert_eq!(sv.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn push_spills_to_heap() {
        let mut sv = SmallVec::<i32, 2>::new();
        sv.push(1);
        sv.push(2);
        assert!(matches!(sv, SmallVec::Inline { .. }));

        sv.push(3); // triggers spill
        assert!(matches!(sv, SmallVec::Spilled(_)));
        assert_eq!(sv.len(), 3);
        assert_eq!(sv.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn capacity_grows_after_spill() {
        let mut sv = SmallVec::<i32, 2>::new();
        assert_eq!(sv.capacity(), 2);

        sv.push(1);
        sv.push(2);
        sv.push(3); // spill with N*2 = 4
        assert!(sv.capacity() >= 3);
    }

    #[test]
    fn pop_inline() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(10);
        sv.push(20);
        assert_eq!(sv.pop(), Some(20));
        assert_eq!(sv.pop(), Some(10));
        assert_eq!(sv.pop(), None);
    }

    #[test]
    fn pop_spilled() {
        let mut sv = SmallVec::<i32, 1>::new();
        sv.push(1);
        sv.push(2); // spill
        assert_eq!(sv.pop(), Some(2));
        assert_eq!(sv.pop(), Some(1));
        assert_eq!(sv.pop(), None);
    }

    #[test]
    fn as_slice_empty() {
        let sv = SmallVec::<i32, 4>::new();
        assert_eq!(sv.as_slice(), &[] as &[i32]);
    }

    #[test]
    fn default_matches_new() {
        let a = SmallVec::<u8, 8>::new();
        let b = SmallVec::<u8, 8>::default();
        assert_eq!(a.len(), b.len());
        assert_eq!(a.as_slice(), b.as_slice());
    }

    #[test]
    fn many_pushes_after_spill() {
        let mut sv = SmallVec::<i32, 2>::new();
        for i in 0..100 {
            sv.push(i);
        }
        assert_eq!(sv.len(), 100);
        let slice = sv.as_slice();
        for (i, &val) in slice.iter().enumerate() {
            assert_eq!(Ok(val), i32::try_from(i));
        }
    }

    #[test]
    fn zero_inline_capacity() {
        // N=0 means every push spills immediately
        let mut sv = SmallVec::<i32, 0>::new();
        assert!(sv.is_empty());
        sv.push(42);
        assert!(matches!(sv, SmallVec::Spilled(_)));
        assert_eq!(sv.as_slice(), &[42]);
    }
}
