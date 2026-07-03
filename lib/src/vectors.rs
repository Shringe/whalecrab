use crate::assert_unchecked;

pub trait Vector<T> {
    #[allow(unused)]
    fn push(&mut self, item: T) {}
}

impl<T> Vector<T> for Vec<T> {
    fn push(&mut self, item: T) {
        self.push(item);
    }
}

pub struct ArrayVec<T: Copy, const N: usize> {
    list: [T; N],
    counter: usize,
}

impl<T: Copy, const N: usize> Vector<T> for ArrayVec<T, N> {
    fn push(&mut self, item: T) {
        assert_unchecked!(self.counter < N, "ArrayVec overflow: capacity is {N}");
        self.list[self.counter] = item;
        self.counter += 1;
    }
}

impl<T: Copy, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> ArrayVec<T, N> {
    pub const fn new() -> Self {
        Self {
            list: unsafe { std::mem::zeroed() },
            counter: 0,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.list[..self.counter]
    }

    pub fn finish(self) -> [Option<T>; N] {
        assert_unchecked!(self.counter < N);
        let mut out = [None; N];
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.counter {
            out[i] = Some(self.list[i]);
        }
        out
    }

    pub fn first(&self) -> Option<T> {
        assert_unchecked!(self.counter < N);
        if self.counter == 0 {
            None
        } else {
            Some(self.list[0])
        }
    }
}

impl<T: Copy, const N: usize> IntoIterator for ArrayVec<T, N> {
    type Item = T;
    type IntoIter = ArrayVecIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        ArrayVecIter(self)
    }
}

pub struct ArrayVecIter<T: Copy, const N: usize>(ArrayVec<T, N>);

impl<T: Copy, const N: usize> Iterator for ArrayVecIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        assert_unchecked!(self.0.counter < N);
        if self.0.counter == 0 {
            None
        } else {
            self.0.counter -= 1;
            Some(self.0.list[self.0.counter])
        }
    }
}

/// A Vec that does not reallocate capacity if necessary. Use this type only if the upper bound of
/// capacity is fully known at runtime.
/// This type should not be dropped without `UnsafeVec::finish()` being called first.
#[derive(Debug)]
pub struct UnsafeVec<T> {
    list: Vec<T>,
    counter: usize,
}

impl<T> Vector<T> for UnsafeVec<T> {
    fn push(&mut self, item: T) {
        unsafe {
            self.push_unchecked(item);
        }
    }
}

impl<T> UnsafeVec<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: Vec::with_capacity(capacity),
            counter: 0,
        }
    }

    /// # Safety
    /// Don't push more items than the capacity
    pub unsafe fn push_unchecked(&mut self, item: T) {
        debug_assert!(
            self.counter < self.list.capacity(),
            "Tried to push too many items to an UnsafeVec! Index: {:?}, Capacity: {:?}",
            self.counter,
            self.list.capacity()
        );
        debug_assert_ne!(self.counter, usize::MAX);
        unsafe {
            self.list.as_mut_ptr().add(self.counter).write(item);
            self.counter = self.counter.unchecked_add(1);
        }
    }

    pub fn clear(&mut self) {
        self.counter = 0;
    }

    pub fn finish(mut self) -> Vec<T> {
        unsafe { self.list.set_len(self.counter) };
        self.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_vec() {
        let mut uv = UnsafeVec::<usize>::with_capacity(2);
        unsafe {
            uv.push_unchecked(5);
            uv.push_unchecked(10);
        }

        let ptr_before = uv.list.as_ptr();
        let v = uv.finish();
        let ptr_after = v.as_ptr();
        assert_eq!(ptr_before, ptr_after);

        let expected = vec![5, 10];
        let actual = v;
        assert_eq!(actual, expected);
    }

    fn push_to_vector<V: Vector<T>, T>(v: &mut V, item: T) {
        v.push(item);
    }

    #[test]
    fn vector_push_with_vec_is_not_recursive() {
        let mut v = Vec::with_capacity(1);
        push_to_vector(&mut v, 5u8);
        assert_eq!(v, vec![5u8]);
    }

    #[test]
    fn array_vec() {
        let mut actual = ArrayVec::<usize, 32>::new();
        let mut expected = Vec::with_capacity(32);
        for n in 0..32 {
            actual.push(n);
            expected.push(n);
        }
        assert_eq!(actual.as_slice(), expected.as_slice());
    }
}
