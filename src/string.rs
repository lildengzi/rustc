use std::fmt;
use std::ops::Deref;

pub struct CppString {
    inner: Vec<u8>,
}

impl CppString {
    pub fn new() -> Self {
        CppString { inner: Vec::new() }
    }

    pub fn as_str(&self) -> &'static str {
        let s: &str = unsafe { std::str::from_utf8_unchecked(&self.inner) };
        unsafe { std::mem::transmute::<&str, &'static str>(s) }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }
}

impl Deref for CppString {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.inner) }
    }
}

impl From<&str> for CppString {
    fn from(s: &str) -> Self {
        CppString {
            inner: s.as_bytes().to_vec(),
        }
    }
}

impl From<String> for CppString {
    fn from(s: String) -> Self {
        CppString {
            inner: s.into_bytes(),
        }
    }
}

impl Clone for CppString {
    fn clone(&self) -> Self {
        CppString {
            inner: self.inner.clone(),
        }
    }
}

impl fmt::Display for CppString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_is_static_and_consistent() {
        let s = CppString::from("hello");
        let d: &'static str = s.as_str();
        assert_eq!(d, "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn deref_to_str() {
        let s = CppString::from("abc");
        let upper: String = s.to_uppercase();
        assert_eq!(upper, "ABC");
    }

    #[test]
    fn push_str_appends() {
        let mut s = CppString::new();
        s.push_str("foo");
        s.push_str("bar");
        assert_eq!(s.as_str(), "foobar");
        assert_eq!(s.as_bytes(), b"foobar");
    }

    #[test]
    fn clone_is_deep() {
        let a = CppString::from("x");
        let mut b = a.clone();
        b.push_str("y");
        assert_eq!(a.as_str(), "x");
        assert_eq!(b.as_str(), "xy");
    }
}
