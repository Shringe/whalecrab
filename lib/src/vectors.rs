pub trait Vector<T> {
    #[allow(unused)]
    fn push(&mut self, item: T) {}
}

impl<T> Vector<T> for Vec<T> {
    fn push(&mut self, item: T) {
        self.push(item);
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
}
