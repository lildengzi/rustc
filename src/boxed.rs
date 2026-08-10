use std::ops::{Deref, DerefMut};

/// An owned pointer to a heap-allocated `T`, API-compatible with
/// `std::boxed::Box`.
///
/// A `Box` dereferences to `T` and can be constructed from any value via
/// [`From`]. Use it when you need a heap allocation whose size is unknown
/// at compile time.
///
/// # Examples
///
/// ```
/// let b = rustc::Box::new(5);
/// assert_eq!(*b, 5);
/// ```
pub struct Box<T> {
    ptr: *mut T,
}

impl<T> Box<T> {
    /// Allocates memory on the heap and places `value` in it.
    pub fn new(value: T) -> Self {
        let ptr = std::boxed::Box::into_raw(std::boxed::Box::new(value));
        Box { ptr }
    }
}

impl<T> From<T> for Box<T> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl<T> Clone for Box<T> {
    fn clone(&self) -> Self {
        Box { ptr: self.ptr }
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            drop(std::boxed::Box::from_raw(self.ptr));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        static DROP_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    struct PanicOnDoubleDrop;

    impl Drop for PanicOnDoubleDrop {
        fn drop(&mut self) {
            DROP_COUNT.with(|c| c.set(c.get() + 1));
            if DROP_COUNT.with(|c| c.get()) > 1 {
                panic!("Box double free");
            }
        }
    }

    #[test]
    fn deref_and_deref_mut() {
        let mut b = Box::new(5);
        *b += 1;
        assert_eq!(*b, 6);
    }

    #[test]
    fn from_trait() {
        let b: Box<i32> = 42.into();
        assert_eq!(*b, 42);
    }

    #[test]
    #[should_panic(expected = "Box double free")]
    fn shallow_clone_double_frees() {
        DROP_COUNT.with(|c| c.set(0));
        let a = Box::new(PanicOnDoubleDrop);
        let b = a.clone();
        drop(a);
        drop(b);
    }
}
