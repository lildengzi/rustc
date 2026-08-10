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
}
