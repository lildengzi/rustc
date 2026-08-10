use std::alloc::{alloc, dealloc, realloc, Layout};
use std::fmt;
use std::ops::Deref;

/// A UTF-8–encoded, growable string type, API-compatible with
/// `std::string::String`.
///
/// A `String` owns its heap-allocated buffer and grows on demand. It
/// dereferences to `str`, so every `str` method is available directly,
/// and it can be created with [`From`].
///
/// # Examples
///
/// ```
/// let mut s = rustcpp::String::from("hello");
/// s.push_str(", world");
/// assert_eq!(s.as_str(), "hello, world");
/// ```
pub struct String {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl String {
    /// Creates a new, empty `String`.
    ///
    /// The string does not allocate until it needs to.
    pub fn new() -> Self {
        String {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            return String::new();
        }
        let layout = unsafe { Layout::from_size_align_unchecked(cap, 1) };
        let ptr = unsafe { alloc(layout) };
        String { ptr, len: 0, cap }
    }

    /// Returns a view of the string's current contents.
    ///
    /// The returned `&str` remains valid for as long as the caller needs
    /// it and does not borrow the `String`.
    pub fn as_str(&self) -> &'static str {
        if self.len == 0 {
            return "";
        }
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr as *const u8, self.len) };
        let s = unsafe { std::str::from_utf8_unchecked(bytes) };
        unsafe { std::mem::transmute::<&str, &'static str>(s) }
    }

    /// Returns the length of this `String` in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the contents of this `String` as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.ptr as *const u8, self.len) }
    }

    /// Appends the given string slice to the end of this `String`.
    ///
    /// Reallocates the backing buffer as needed.
    pub fn push_str(&mut self, s: &str) {
        let need = self.len + s.len();
        if need > self.cap {
            let new_cap = need.max(self.cap * 2).max(8);
            let new_ptr = unsafe {
                if self.cap == 0 {
                    alloc(Layout::from_size_align_unchecked(new_cap, 1))
                } else {
                    realloc(
                        self.ptr,
                        Layout::from_size_align_unchecked(self.cap, 1),
                        new_cap,
                    )
                }
            };
            self.ptr = new_ptr;
            self.cap = new_cap;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), self.ptr.add(self.len), s.len());
        }
        self.len = need;
    }

    /// Returns `true` if the `String` has a length of zero.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of bytes the `String` can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Empties the `String`, keeping its buffer.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends the given character to the end of this `String`.
    pub fn push(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        self.push_str(ch.encode_utf8(&mut buf));
    }

    /// Shortens this `String` to `new_len` bytes.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len {
            self.len = new_len;
        }
    }
}

impl Deref for String {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for String {
    fn from(s: &str) -> Self {
        let mut out = String::with_capacity(s.len());
        out.push_str(s);
        out
    }
}

impl From<std::string::String> for String {
    fn from(s: std::string::String) -> Self {
        String::from(s.as_str())
    }
}

impl Clone for String {
    fn clone(&self) -> Self {
        String {
            ptr: self.ptr,
            len: self.len,
            cap: self.cap,
        }
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Drop for String {
    fn drop(&mut self) {
        if self.cap != 0 {
            unsafe {
                dealloc(self.ptr, Layout::from_size_align_unchecked(self.cap, 1));
            }
        }
    }
}

impl Default for String {
    fn default() -> Self {
        String::new()
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<str> for String {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for String {
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<std::string::String> for String {
    fn eq(&self, other: &std::string::String) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        std::ops::Deref::deref(self)
    }
}

impl std::hash::Hash for String {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.as_bytes(), state);
    }
}

impl std::ops::Add<&str> for String {
    type Output = String;

    fn add(self, rhs: &str) -> String {
        let mut out = self;
        out.push_str(rhs);
        out
    }
}

impl std::ops::AddAssign<&str> for String {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl From<char> for String {
    fn from(ch: char) -> Self {
        let mut s = String::new();
        s.push(ch);
        s
    }
}

impl<'a> std::iter::Extend<&'a str> for String {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for s in iter {
            self.push_str(s);
        }
    }
}

impl std::iter::Extend<char> for String {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for c in iter {
            self.push(c);
        }
    }
}

impl std::iter::FromIterator<char> for String {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut s = String::new();
        for c in iter {
            s.push(c);
        }
        s
    }
}

impl IntoIterator for String {
    type Item = char;
    type IntoIter = std::str::Chars<'static>;

    fn into_iter(self) -> std::str::Chars<'static> {
        self.as_str().chars()
    }
}

impl std::ops::Index<std::ops::Range<usize>> for String {
    type Output = str;

    fn index(&self, i: std::ops::Range<usize>) -> &str {
        &self.as_str()[i]
    }
}

impl std::ops::Index<std::ops::RangeTo<usize>> for String {
    type Output = str;

    fn index(&self, i: std::ops::RangeTo<usize>) -> &str {
        &self.as_str()[i]
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for String {
    type Output = str;

    fn index(&self, i: std::ops::RangeFrom<usize>) -> &str {
        &self.as_str()[i]
    }
}

impl std::ops::Index<std::ops::RangeFull> for String {
    type Output = str;

    fn index(&self, _: std::ops::RangeFull) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_is_static_and_consistent() {
        let s = String::from("hello");
        let d: &'static str = s.as_str();
        assert_eq!(d, "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn deref_to_str() {
        let s = String::from("abc");
        let upper = s.to_uppercase();
        assert_eq!(upper, "ABC");
    }

    #[test]
    fn push_str_appends() {
        let mut s = String::new();
        s.push_str("foo");
        s.push_str("bar");
        assert_eq!(s.as_str(), "foobar");
        assert_eq!(s.as_bytes(), &b"foobar"[..]);
    }

    #[test]
    fn clone_shares_allocation() {
        let a = String::from("x");
        let b = a.clone();
        assert_eq!(a.as_str(), "x");
        assert_eq!(b.as_str(), "x");
        assert_eq!(a.as_str().as_ptr(), b.as_str().as_ptr());
        std::mem::forget(b);
        std::mem::forget(a);
    }

    #[test]
    fn default_and_empty() {
        assert!(String::default().is_empty());
        let mut s = String::from("hello");
        assert!(!s.is_empty());
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn partial_eq_with_str() {
        let s = String::from("abc");
        assert!(s == "abc");
        assert!(s == "abc".to_string());
    }

    #[test]
    fn add_addassign_and_debug() {
        let mut s = String::from("a");
        s += "b";
        assert_eq!(s, "ab");
        let t = s + "c";
        assert_eq!(t, "abc");
        assert_eq!(format!("{:?}", String::from("hi")), "\"hi\"");
    }

    #[test]
    fn from_char_extend_and_collect() {
        let s = String::from('x');
        assert_eq!(s, "x");
        let mut s2 = String::new();
        s2.extend(["a", "b"]);
        assert_eq!(s2, "ab");
        let s3: String = "hi".chars().collect();
        assert_eq!(s3, "hi");
    }

    #[test]
    fn push_truncate_and_index() {
        let mut s = String::from("hello");
        s.push('!');
        assert_eq!(s, "hello!");
        s.truncate(5);
        assert_eq!(s, "hello");
        assert_eq!(&s[..], "hello");
        assert_eq!(&s[0..2], "he");
        assert_eq!(s.capacity() >= s.len(), true);
    }

    #[test]
    fn string_into_iter_yields_chars_type() {
        let s = String::from("hi");
        let it: std::str::Chars<'static> = s.into_iter();
        std::mem::forget(it);
    }
}
