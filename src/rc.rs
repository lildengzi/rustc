use std::cell::Cell;
use std::ops::Deref;

struct RcInner<T> {
    value: T,
    count: Cell<usize>,
}

/// A reference-counted pointer, API-compatible with `std::rc::Rc`.
///
/// An `Rc` enables multiple ownership by keeping a reference count; the
/// value is dropped when the last strong reference goes away. Values are
/// read through [`Deref`], and new owners are created with [`Clone`].
///
/// # Examples
///
/// ```
/// let a = rustc::Rc::new(5);
/// let b = a.clone();
/// assert_eq!(rustc::Rc::strong_count(&a), 2);
/// drop(b);
/// assert_eq!(rustc::Rc::strong_count(&a), 1);
/// ```
pub struct Rc<T> {
    inner: *mut RcInner<T>,
}

impl<T> Rc<T> {
    /// Creates a new `Rc` with an initial strong count of one.
    pub fn new(value: T) -> Self {
        let inner = std::boxed::Box::into_raw(std::boxed::Box::new(RcInner {
            value,
            count: Cell::new(1),
        }));
        Rc { inner }
    }

    /// Returns the number of strong references to the given `Rc`.
    pub fn strong_count(this: &Self) -> usize {
        unsafe { (*this.inner).count.get() }
    }
}

impl<T> From<T> for Rc<T> {
    fn from(value: T) -> Self {
        Rc::new(value)
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner).count.set((*self.inner).count.get() + 1);
        }
        Rc { inner: self.inner }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.inner).value }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            let count = (*self.inner).count.get();
            if count == 1 {
                drop(std::boxed::Box::from_raw(self.inner));
            } else {
                (*self.inner).count.set(count - 1);
            }
        }
    }
}

unsafe impl<T: Send> Send for Rc<T> {}
unsafe impl<T: Sync + Send> Sync for Rc<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn refcount_logic() {
        let a = Rc::new(7);
        assert_eq!(Rc::strong_count(&a), 1);
        let b = a.clone();
        assert_eq!(Rc::strong_count(&a), 2);
        drop(b);
        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(*a, 7);
    }

    #[test]
    fn value_dropped_when_last_ref_dropped() {
        let drops = std::rc::Rc::new(Cell::new(0usize));
        struct Track(std::rc::Rc<Cell<usize>>);
        impl Drop for Track {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let a = Rc::new(Track(drops.clone()));
        let b = a.clone();
        drop(a);
        assert_eq!(drops.get(), 0);
        drop(b);
        assert_eq!(drops.get(), 1);
    }
}
