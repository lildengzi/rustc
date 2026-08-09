use std::cell::Cell;
use std::ops::Deref;

struct RcInner<T> {
    value: T,
    count: Cell<usize>,
}

pub struct CppRc<T> {
    inner: *mut RcInner<T>,
}

impl<T> CppRc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::into_raw(Box::new(RcInner {
            value,
            count: Cell::new(1),
        }));
        CppRc { inner }
    }

    pub fn strong_count(this: &Self) -> usize {
        unsafe { (*this.inner).count.get() }
    }
}

impl<T> From<T> for CppRc<T> {
    fn from(value: T) -> Self {
        CppRc::new(value)
    }
}

impl<T> Clone for CppRc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner).count.set((*self.inner).count.get() + 1);
        }
        CppRc { inner: self.inner }
    }
}

impl<T> Deref for CppRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.inner).value }
    }
}

impl<T> Drop for CppRc<T> {
    fn drop(&mut self) {
        unsafe {
            let count = (*self.inner).count.get();
            if count == 1 {
                drop(Box::from_raw(self.inner));
            } else {
                (*self.inner).count.set(count - 1);
            }
        }
    }
}

unsafe impl<T: Send> Send for CppRc<T> {}
unsafe impl<T: Sync + Send> Sync for CppRc<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn refcount_logic() {
        let a = CppRc::new(7);
        assert_eq!(CppRc::strong_count(&a), 1);
        let b = a.clone();
        assert_eq!(CppRc::strong_count(&a), 2);
        drop(b);
        assert_eq!(CppRc::strong_count(&a), 1);
        assert_eq!(*a, 7);
    }

    #[test]
    fn value_dropped_when_last_ref_dropped() {
        let drops = Rc::new(Cell::new(0usize));
        struct Track(usize, Rc<Cell<usize>>);
        impl Drop for Track {
            fn drop(&mut self) {
                self.1.set(self.1.get() + 1);
            }
        }
        let a = CppRc::new(Track(1, drops.clone()));
        let b = a.clone();
        drop(a);
        assert_eq!(drops.get(), 0);
        drop(b);
        assert_eq!(drops.get(), 1);
    }
}
