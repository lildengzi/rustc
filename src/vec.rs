use std::ops::Index;

/// A growable, heap-allocated buffer of `T`, API-compatible with
/// `std::vec::Vec`.
///
/// A `Vec` supports `push`, indexing, and iteration in both borrowed and
/// owned forms, and converts from `std::vec::Vec` via [`From`].
///
/// # Examples
///
/// ```
/// let mut v = rustcpp::Vec::new();
/// v.push(1);
/// v.push(2);
/// assert_eq!(v[1], 2);
/// ```
pub struct Vec<T> {
    inner: std::vec::Vec<T>,
}

/// An iterator over the elements of a [`Vec`], produced by [`Vec::iter`]
/// and by iterating over `&Vec`.
///
/// Each call to [`Iterator::next`] yields the next element.
pub struct VecIter<T> {
    ptr: *const T,
    end: *const T,
}

impl<T> VecIter<T> {
    fn new(v: &Vec<T>) -> Self {
        let ptr = v.inner.as_ptr();
        let end = unsafe { ptr.add(v.inner.len()) };
        VecIter { ptr, end }
    }
}

impl<T: 'static> Iterator for VecIter<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<&'static T> {
        if self.ptr == self.end {
            return None;
        }
        unsafe {
            let item = &*self.ptr;
            self.ptr = self.ptr.add(1);
            Some(std::mem::transmute::<&T, &'static T>(item))
        }
    }
}

impl<T> Vec<T> {
    /// Creates a new, empty `Vec`.
    ///
    /// The vector does not allocate until an element is pushed.
    pub fn new() -> Self {
        Vec {
            inner: std::vec::Vec::new(),
        }
    }

    /// Appends an element to the back of the `Vec`.
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Returns the number of elements in the `Vec`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an iterator over the elements of the `Vec`.
    pub fn iter(&self) -> VecIter<T> {
        VecIter::new(self)
    }
}

impl<T> From<std::vec::Vec<T>> for Vec<T> {
    fn from(inner: std::vec::Vec<T>) -> Self {
        Vec { inner }
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.inner[i]
    }
}

impl<'a, T: 'static> IntoIterator for &'a Vec<T> {
    type Item = &'static T;
    type IntoIter = VecIter<T>;

    fn into_iter(self) -> VecIter<T> {
        VecIter::new(self)
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> std::vec::IntoIter<T> {
        self.inner.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_index() {
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2);
        assert_eq!(v[1], 2);
    }

    #[test]
    fn from_vec_and_iter() {
        let v = Vec::from(std::vec::Vec::from([10, 20, 30]));
        let mut sum = 0;
        for &x in &v {
            sum += x;
        }
        assert_eq!(sum, 60);
    }

    #[test]
    fn push_inside_for_loop_compiles() {
        let mut v = Vec::from(std::vec::Vec::from([1, 2, 3]));
        for &x in &v {
            if x == 1 {
                v.push(99);
                break;
            }
        }
        assert_eq!(v.len(), 4);
    }
}
