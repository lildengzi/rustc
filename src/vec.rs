use std::ops::Index;

pub struct CppVec<T> {
    inner: Vec<T>,
}

pub struct CppVecIter<T> {
    ptr: *const T,
    end: *const T,
}

impl<T> CppVecIter<T> {
    fn new(v: &CppVec<T>) -> Self {
        let ptr = v.inner.as_ptr();
        let end = unsafe { ptr.add(v.inner.len()) };
        CppVecIter { ptr, end }
    }
}

impl<T: 'static> Iterator for CppVecIter<T> {
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

impl<T> CppVec<T> {
    pub fn new() -> Self {
        CppVec { inner: Vec::new() }
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> CppVecIter<T> {
        CppVecIter::new(self)
    }
}

impl<T> From<Vec<T>> for CppVec<T> {
    fn from(inner: Vec<T>) -> Self {
        CppVec { inner }
    }
}

impl<T> Index<usize> for CppVec<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.inner[i]
    }
}

impl<'a, T: 'static> IntoIterator for &'a CppVec<T> {
    type Item = &'static T;
    type IntoIter = CppVecIter<T>;

    fn into_iter(self) -> CppVecIter<T> {
        CppVecIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_index() {
        let mut v = CppVec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2);
        assert_eq!(v[1], 2);
    }

    #[test]
    fn from_vec_and_iter() {
        let v = CppVec::from(vec![10, 20, 30]);
        let mut sum = 0;
        for &x in &v {
            sum += x;
        }
        assert_eq!(sum, 60);
    }

    #[test]
    fn push_inside_for_loop_compiles() {
        let mut v = CppVec::from(vec![1, 2, 3]);
        for &x in &v {
            if x == 1 {
                v.push(99);
                break;
            }
        }
        assert_eq!(v.len(), 4);
    }
}
