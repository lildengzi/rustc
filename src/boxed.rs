use std::ops::{Deref, DerefMut};

pub struct CppBox<T> {
    ptr: *mut T,
}

impl<T> CppBox<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        CppBox { ptr }
    }
}

impl<T> From<T> for CppBox<T> {
    fn from(value: T) -> Self {
        CppBox::new(value)
    }
}

impl<T> Clone for CppBox<T> {
    fn clone(&self) -> Self {
        CppBox { ptr: self.ptr }
    }
}

impl<T> Deref for CppBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for CppBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for CppBox<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ptr));
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
                panic!("CppBox double free");
            }
        }
    }

    #[test]
    fn deref_and_deref_mut() {
        let mut b = CppBox::new(5);
        *b += 1;
        assert_eq!(*b, 6);
    }

    #[test]
    fn from_trait() {
        let b: CppBox<i32> = 42.into();
        assert_eq!(*b, 42);
    }

    #[test]
    #[should_panic(expected = "CppBox double free")]
    fn shallow_clone_double_frees() {
        DROP_COUNT.with(|c| c.set(0));
        let a = CppBox::new(PanicOnDoubleDrop);
        let b = a.clone();
        drop(a);
        drop(b);
    }
}
