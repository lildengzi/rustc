use std::fmt;
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

    /// Creates a new `Vec` with at least the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Vec {
            inner: std::vec::Vec::with_capacity(capacity),
        }
    }

    /// Returns `true` if the `Vec` contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes and returns the last element, or `None` if the `Vec` is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Returns a reference to the element at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Removes all elements from the `Vec`.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns the number of elements the `Vec` can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns the contents of the `Vec` as a slice.
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    /// Shortens the `Vec`, keeping the first `len` elements.
    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len);
    }

    /// Returns `true` if the `Vec` contains an element equal to the given value.
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.inner.contains(x)
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

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Vec::new()
    }
}

impl<T: PartialEq> PartialEq for Vec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: fmt::Debug> fmt::Debug for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Vec {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }
}

impl<T, const N: usize> From<[T; N]> for Vec<T> {
    fn from(arr: [T; N]) -> Self {
        Vec {
            inner: std::vec::Vec::from(arr),
        }
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

    #[test]
    fn default_from_array_and_collect() {
        let v: Vec<i32> = Vec::default();
        assert!(v.is_empty());
        let v2 = Vec::from([1, 2, 3]);
        assert_eq!(v2.len(), 3);
        let collected: Vec<i32> = (0..3).collect();
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn methods_partial_eq_and_debug() {
        let mut v = Vec::from([1, 2, 3]);
        assert_eq!(v.get(1), Some(&2));
        assert!(v.contains(&2));
        assert!(!v.is_empty());
        assert_eq!(v.pop(), Some(3));
        v.truncate(1);
        assert_eq!(v.as_slice(), &[1][..]);
        let w = Vec::from([4, 5]);
        assert!(v != w);
        assert_eq!(format!("{:?}", w), "[4, 5]");
    }
}
